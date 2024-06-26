use opentelemetry::trace::{SpanId, TraceId};
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::{global, Context, KeyValue};
use opentelemetry_otlp::{new_exporter, HasExportConfig, WithExportConfig};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{config, BatchConfigBuilder, Sampler};
use opentelemetry_sdk::Resource;
use pg_ferret_shared::{Pid, ThreadId};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tonic::metadata::{MetadataMap, MetadataValue};

#[derive(Clone)]
pub struct TraceEmitter {
    tracer: opentelemetry_sdk::trace::Tracer,
    contexts: ThreadContexts,
    trace_ids: TraceIds,
    remote_contexts: RemoteContexts,
}

type ThreadContexts = Arc<Mutex<HashMap<u32, Vec<(String, Context)>>>>;
type TraceIds = Arc<Mutex<HashMap<u32, u128>>>;
type RemoteContexts = Arc<Mutex<HashMap<u32, Context>>>;

lazy_static::lazy_static! {
    /// The tonic OTLP exporter requires that the metadata headers are static, so
    /// we need to load them at startup and store them in a static variable.
    static ref OTEL_EXPORTER_OTLP_HEADERS: Option<String> = {
        env::var("OTEL_EXPORTER_OTLP_HEADERS").ok()
    };
    static ref METADATA: MetadataMap = {
        let mut metadata = MetadataMap::new();
        if let Some(headers) = &*OTEL_EXPORTER_OTLP_HEADERS {
            for header in headers.split(',') {
                let mut parts = header.split('=');
                let key = parts.next().unwrap();
                let value = parts.next().unwrap();
                metadata.insert(key, MetadataValue::from_str(value).unwrap());
            }
        }
        println!("Using {} headers for OTLP", metadata.len());
        metadata
    };
}

impl TraceEmitter {
    pub fn initialise(endpoint: Option<String>) -> Result<Self, anyhow::Error> {
        let endpoint = match endpoint {
            Some(endpoint) => {
                println!("Sending traces to {}", endpoint);
                endpoint
            }
            None => {
                println!("Sending traces to built-in backend");
                "http://localhost:4317".to_string()
            }
        };
        let mut exporter = new_exporter()
            .tonic()
            .with_endpoint(endpoint)
            .with_metadata(METADATA.clone());
        println!(
            "Using {:?} protocol for OTLP",
            exporter.export_config().protocol
        );

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(
                config()
                    .with_resource(Resource::new(vec![KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        "postgres",
                    )]))
                    .with_sampler(Sampler::AlwaysOn),
            )
            .with_batch_config(
                BatchConfigBuilder::default()
                    .with_max_concurrent_exports(10)
                    .with_max_export_batch_size(10000)
                    .build(),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        global::set_text_map_propagator(TraceContextPropagator::new());

        let contexts: ThreadContexts = Arc::new(Mutex::new(HashMap::new()));
        let trace_ids: TraceIds = Arc::new(Mutex::new(HashMap::new()));
        let remote_contexts: RemoteContexts = Arc::new(Mutex::new(HashMap::new()));

        let tracing = Self {
            tracer,
            contexts,
            trace_ids,
            remote_contexts,
        };
        {
            let tracing = tracing.clone();
            tokio::spawn(async move {
                tracing.record_metrics().await;
            });
        }

        Ok(tracing)
    }

    async fn record_metrics(&self) {
        let meter = global::meter("tracing_metadata");
        let tracing_contexts_count = meter
            .u64_gauge("pg_ferret_tracing_contexts_count")
            .with_description("The number of active tracing contexts within pg ferret")
            .init();
        let tracing_trace_ids_count = meter
            .u64_gauge("pg_ferret_tracing_trace_ids_count")
            .with_description("The number of active trace ids within pg ferret")
            .init();
        let tracing_remote_contexts_count = meter
            .u64_gauge("pg_ferret_tracing_remote_contexts_count")
            .with_description("The number of active remote contexts within pg ferret")
            .init();

        loop {
            {
                let contexts = self.contexts.lock().unwrap();
                let trace_ids = self.trace_ids.lock().unwrap();
                let remote_contexts = self.remote_contexts.lock().unwrap();
                tracing_contexts_count.record(contexts.len() as u64, &[]);
                tracing_trace_ids_count.record(trace_ids.len() as u64, &[]);
                tracing_remote_contexts_count.record(remote_contexts.len() as u64, &[]);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start_span(
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
                Some(parent) => builder.start_with_context(&self.tracer, parent),
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
                let span = builder.start_with_context(&self.tracer, parent_context);
                Context::current_with_span(span)
            });
            if let Some(context) = context {
                thread_contexts.push((name.to_string(), context));
            }
        };
    }

    pub fn end_span(&self, name: &str, thread_id: ThreadId) {
        let mut contexts = self.contexts.lock().unwrap();
        let thread_contexts = contexts.entry(thread_id).or_default();
        let last = thread_contexts.pop();
        if let Some((n, context)) = last {
            if n == name {
                context.span().end();
            } else {
                thread_contexts.push((n, context));
            }
        }
    }
}
