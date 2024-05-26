use aya::maps::{AsyncPerfEventArray, MapData};
use aya::programs::UProbe;
use aya::util::online_cpus;
use aya::{include_bytes_aligned, Bpf};
use aya_log::BpfLogger;
use bytes::BytesMut;
use clap::Parser;
use log::{debug, info, warn};
use opentelemetry::global::ObjectSafeTracerProvider;
use opentelemetry::trace::TracerProvider;
use opentelemetry::trace::{SpanId, TraceId};
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::{global, Context, KeyValue};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{BatchConfigBuilder, BatchSpanProcessor, Sampler};
use opentelemetry_sdk::{runtime, Resource};
use pg_ferret_shared::{Event, Pid, PostgresEntry, ThreadId};
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::signal;

mod generated;
use generated::{POSTGRES_FUNCS, POSTGRES_TOP_LEVEL_COMMAND_FUNCS};

#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long)]
    pid: Option<i32>,
}

type ThreadContexts = Arc<Mutex<HashMap<u32, Vec<(String, Context)>>>>;
type TraceIds = Arc<Mutex<HashMap<u32, u128>>>;
type RemoteContexts = Arc<Mutex<HashMap<u32, Context>>>;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("Starting pg-ferret userspace collector");
    //let opt = Opt::parse();

    env_logger::init();

    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {}", ret);
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/kernelspace-ebpf-tracer"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/kernelspace-ebpf-tracer"
    ))?;
    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        warn!("failed to initialize eBPF logger: {}", e);
    }

    attach_uprobes(&mut bpf).unwrap();

    let tracing = Tracing::initialise()?;

    let mut perf_array: AsyncPerfEventArray<MapData> =
        AsyncPerfEventArray::try_from(bpf.take_map("EVENTS").unwrap())?;

    let len_of_data = std::mem::size_of::<Event>();
    for cpu_id in online_cpus()? {
        let mut buf = perf_array.open(cpu_id, Some(32))?;
        let tracing = tracing.clone();

        tokio::spawn(async move {
            let mut buffers = (0..10)
                .map(|_| BytesMut::with_capacity(len_of_data))
                .collect::<Vec<_>>();

            loop {
                //println!("Waiting for next event");
                let events = match buf.read_events(&mut buffers).await {
                    Ok(events) => events,
                    Err(e) => {
                        warn!("Error reading events: {}", e);
                        continue;
                    }
                };
                //println!("Got {} events", events.read);

                #[allow(clippy::needless_range_loop)]
                for i in 0..events.read {
                    let buf = &mut buffers[i];
                    let data = buf.as_ptr() as *const Event;
                    let event = unsafe { *data };

                    match event {
                        Event::Entry(entry) => match entry {
                            PostgresEntry::ExecSimpleQuery(query, thread_id, pid) => {
                                let query: String = utf8_array_to_string(&query);
                                let (query, remote_parent) = extract_span_context(query);
                                tracing.start_span(
                                    "exec_simple_query",
                                    thread_id,
                                    Some(pid),
                                    true,
                                    true,
                                    true,
                                    remote_parent,
                                    vec![
                                        KeyValue::new("query", query),
                                        KeyValue::new("pid", pid.to_string()),
                                    ],
                                );
                            }
                            PostgresEntry::ExecParseMessage(query, thread_id, pid) => {
                                let query: String = utf8_array_to_string(&query);
                                let (query, remote_parent) = extract_span_context(query);
                                tracing.start_span(
                                    "exec_parse_message",
                                    thread_id,
                                    Some(pid),
                                    true,
                                    true,
                                    false,
                                    remote_parent,
                                    vec![
                                        KeyValue::new("query", query),
                                        KeyValue::new("pid", pid.to_string()),
                                    ],
                                );
                            }
                            PostgresEntry::ExecBindMessage(thread_id, pid) => {
                                tracing.start_span(
                                    "exec_bind_message",
                                    thread_id,
                                    Some(pid),
                                    true,
                                    false,
                                    false,
                                    None,
                                    vec![],
                                );
                            }
                            PostgresEntry::ExecExecuteMessage(thread_id, pid) => {
                                tracing.start_span(
                                    "exec_execute_message",
                                    thread_id,
                                    Some(pid),
                                    true,
                                    false,
                                    true,
                                    None,
                                    vec![],
                                );
                            }
                            PostgresEntry::Other(func, thread_id) => {
                                let func: String = utf8_array_to_string(&func);
                                tracing.start_span(
                                    &func,
                                    thread_id,
                                    None,
                                    false,
                                    false,
                                    false,
                                    None,
                                    vec![],
                                );
                            }
                        },
                        Event::Return(func, thread_id) => {
                            let func: String = utf8_array_to_string(&func);
                            tracing.end_span(&func, thread_id);
                        }
                    }
                }
            }
        });
    }

    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}

#[derive(Clone)]
struct Tracing {
    tracer: opentelemetry_sdk::trace::Tracer,
    contexts: ThreadContexts,
    trace_ids: TraceIds,
    remote_contexts: RemoteContexts,
}

impl Tracing {
    fn initialise() -> Result<Self, anyhow::Error> {
        // Stdout tracer
        // let exporter = opentelemetry_stdout::SpanExporter::default();
        // let processor = BatchSpanProcessor::builder(exporter, runtime::Tokio).build();
        // let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        //     .with_span_processor(processor)
        //     .build();
        // let tracer = provider.tracer("pg-ferret");

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint("http://localhost:4317"),
            )
            .with_trace_config(
                opentelemetry_sdk::trace::config()
                    .with_resource(Resource::new(vec![KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        "postgres",
                    )]))
                    .with_sampler(Sampler::AlwaysOn),
            )
            .with_batch_config(
                BatchConfigBuilder::default()
                    .with_max_concurrent_exports(5)
                    .with_max_export_batch_size(2000)
                    .build(),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        global::set_text_map_propagator(TraceContextPropagator::new());

        let contexts: ThreadContexts = Arc::new(Mutex::new(HashMap::new()));
        let trace_ids: TraceIds = Arc::new(Mutex::new(HashMap::new()));
        let remote_contexts: RemoteContexts = Arc::new(Mutex::new(HashMap::new()));
        Ok(Self {
            tracer,
            contexts,
            trace_ids,
            remote_contexts,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn start_span(
        &self,
        name: &str,
        thread_id: ThreadId,
        pid: Option<Pid>,
        top_level: bool,
        first: bool,
        last: bool,
        remote_parent: Option<Context>,
        attributes: Vec<KeyValue>,
    ) {
        let span_id = rand::random::<u64>();
        let mut contexts = self.contexts.lock().unwrap();
        let thread_contexts = contexts.entry(thread_id).or_default();
        let mut remote_contexts = self.remote_contexts.lock().unwrap();

        let mut builder = self
            .tracer
            .span_builder(name.to_string())
            .with_span_id(SpanId::from_bytes(span_id.to_be_bytes()));
        if !attributes.is_empty() {
            builder = builder.with_attributes(attributes);
        }

        if top_level {
            if first {
                if let Some(pid) = pid {
                    remote_contexts.remove(&pid);
                    if let Some(parent_context) = remote_parent {
                        remote_contexts.insert(pid, parent_context);
                    }
                }
            }
            let parent = if let Some(pid) = pid {
                remote_contexts.get(&pid)
            } else {
                None
            };
            let span = match parent {
                Some(parent) => {
                    println!("Starting with parent context");
                    builder.start_with_context(&self.tracer, parent)
                }
                None => {
                    let trace_id = if first {
                        let trace_id = rand::random::<u128>();
                        if let Some(pid) = pid {
                            if !last {
                                let mut trace_ids = self.trace_ids.lock().unwrap();
                                trace_ids.insert(pid, trace_id);
                            }
                        }
                        trace_id
                    } else {
                        pid.and_then(|pid| {
                            let mut trace_ids = self.trace_ids.lock().unwrap();
                            if last {
                                remote_contexts.remove(&pid);
                                trace_ids.remove(&pid)
                            } else {
                                trace_ids.get(&pid).copied()
                            }
                        })
                        .unwrap_or(rand::random::<u128>())
                    };
                    println!("Starting top level span: {}", name);
                    builder
                        .with_trace_id(TraceId::from_bytes(trace_id.to_be_bytes()))
                        .start(&self.tracer)
                }
            };
            if last {
                if let Some(pid) = pid {
                    remote_contexts.remove(&pid);
                }
            }
            let context = Context::current_with_span(span);
            *thread_contexts = vec![(name.to_string(), context)];
        } else {
            let parent_context = thread_contexts.last();
            let context = parent_context.map(|(_, parent_context)| {
                println!("Starting lower level span: {}", name);
                let span = builder.start_with_context(&self.tracer, parent_context);
                Context::current_with_span(span)
            });
            if let Some(context) = context {
                thread_contexts.push((name.to_string(), context));
            }
        };
    }

    fn end_span(&self, name: &str, thread_id: ThreadId) {
        let mut contexts = self.contexts.lock().unwrap();
        let thread_contexts = contexts.entry(thread_id).or_default();
        let last = thread_contexts.pop();
        if let Some((n, context)) = last {
            if n == name {
                println!("Ending span: {}", name);
                context.span().end();
            } else {
                thread_contexts.push((n, context));
            }
        }
    }
}

fn attach_uprobes(bpf: &mut Bpf) -> Result<(), Box<dyn Error>> {
    for target in POSTGRES_TOP_LEVEL_COMMAND_FUNCS {
        let program: &mut UProbe = bpf
            .program_mut(&format!("{}_entry", target))
            .unwrap()
            .try_into()?;
        program.load()?;
        program.attach(Some(target), 0, "/usr/lib/postgresql/16/bin/postgres", None)?;
        let program: &mut UProbe = bpf
            .program_mut(&format!("{}_return", target))
            .unwrap()
            .try_into()?;
        program.load()?;
        program.attach(Some(target), 0, "/usr/lib/postgresql/16/bin/postgres", None)?;
    }
    for target in POSTGRES_FUNCS {
        let program: &mut UProbe = bpf
            .program_mut(&format!("{}_entry", target))
            .unwrap()
            .try_into()?;
        program.load()?;
        program.attach(Some(target), 0, "/usr/lib/postgresql/16/bin/postgres", None)?;
        let program: &mut UProbe = bpf
            .program_mut(&format!("{}_return", target))
            .unwrap()
            .try_into()?;
        program.load()?;
        program.attach(Some(target), 0, "/usr/lib/postgresql/16/bin/postgres", None)?;
    }
    Ok(())
}

fn utf8_array_to_string(array: &[u8]) -> String {
    String::from_utf8_lossy(remove_trailing_nulls(array)).to_string()
}

fn remove_trailing_nulls(bytes: &[u8]) -> &[u8] {
    let mut len = bytes.len();
    while len > 0 && bytes[len - 1] == 0 {
        len -= 1;
    }
    &bytes[..len]
}

fn extract_span_context(query: String) -> (String, Option<Context>) {
    let pattern = r"/\*span=(.+)\*/";
    let re = Regex::new(pattern).unwrap();

    if let Some(caps) = re.captures(&query) {
        if let Some(span_context_str) = caps.get(1) {
            let span_context_str = span_context_str.as_str();
            let remainder = query.replace(&caps[0], "");
            let carrier: std::collections::HashMap<String, String> =
                serde_json::from_str(span_context_str).unwrap();
            let span_context =
                global::get_text_map_propagator(|propagator| propagator.extract(&carrier));
            return (remainder, Some(span_context));
        }
    }

    (query, None)
}
