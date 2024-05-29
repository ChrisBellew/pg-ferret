use crate::tracing::TraceEmitter;
use aya::maps::{AsyncPerfEventArray, MapData};
use bytes::BytesMut;
use log::warn;
use opentelemetry::{global, Context, KeyValue};
use pg_ferret_shared::{Event, PostgresEntry};
use regex::Regex;
use std::{error::Error, mem::size_of};

/// Listen to events coming from our eBPF program on a specific CPU.
pub fn listen_to_cpu(
    cpu_id: u32,
    queue: &mut AsyncPerfEventArray<MapData>,
    tracing: TraceEmitter,
) -> Result<(), Box<dyn Error>> {
    let mut buf = queue.open(cpu_id, Some(64))?;

    tokio::spawn(async move {
        let mut buffers = (0..64)
            .map(|_| BytesMut::with_capacity(size_of::<Event>()))
            .collect::<Vec<_>>();

        loop {
            let events = match buf.read_events(&mut buffers).await {
                Ok(events) => events,
                Err(e) => {
                    warn!("Error reading events: {}", e);
                    continue;
                }
            };

            for buffer in buffers.iter_mut().take(events.read) {
                let data = buffer.as_ptr() as *const Event;

                // SAFETY: We control the buffer that this pointer is dereferenced from
                // and we will only read from it once. The buffer will stay in scope
                // while the event is being processed. Once the event is processed, the
                // buffer will be overwritten by the next event.
                let event = unsafe { *data };

                receive_event_from_bpf(event, &tracing);
            }
        }
    });
    Ok(())
}

// Process an event received from the eBPF program. Start or end a span based on the event type.
pub fn receive_event_from_bpf(event: Event, tracing: &TraceEmitter) {
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
                tracing.start_span(&func, thread_id, None, false, false, false, None, vec![]);
            }
        },
        Event::Return(func, thread_id) => {
            let func: String = utf8_array_to_string(&func);
            tracing.end_span(&func, thread_id);
        }
    }
}

fn utf8_array_to_string(array: &[u8]) -> String {
    String::from_utf8_lossy(remove_trailing_nulls(array)).to_string()
}

/// Helper function to extract the string portion of a null terminated string from a byte array
fn remove_trailing_nulls(bytes: &[u8]) -> &[u8] {
    let mut len = bytes.len();
    while len > 0 && bytes[len - 1] == 0 {
        len -= 1;
    }
    &bytes[..len]
}

/// Extract the span context from a query string, if present, and return the remainder of the
/// query and the span context. The span context is expected to be in the format `/*span=...*/`.
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
