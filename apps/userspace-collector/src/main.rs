use aya::maps::{AsyncPerfEventArray, MapData};
use aya::util::online_cpus;
use bpf::{attach_uprobes, init_bpf};
use log::info;
use receive::listen_to_cpu;
use std::error::Error;
use tokio::signal;
use tracing::TraceEmitter;

mod bpf;
mod generated;
mod receive;
mod tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Starting pg-ferret userspace collector");

    // Initialise the eBPF program and give it to the kernel
    // to attach to the functions we want to trace in postgres.
    let mut bpf = init_bpf()?;
    attach_uprobes(&mut bpf).unwrap();

    // Initialise the tracing system. We'll build traces from the
    // postgres events and send them to the tracing backend using OpenTelemetry.
    let tracing = TraceEmitter::initialise()?;

    // Create a queue to receive events from the eBPF program.
    let mut queue: AsyncPerfEventArray<MapData> =
        AsyncPerfEventArray::try_from(bpf.take_map("EVENTS").unwrap())?;

    // Listen to events from all online CPUs. eBPF hooks will be
    // executed on the CPU that the traced function is running on.
    // This helps us to avoid context switches and cache misses.
    let cpus = online_cpus()?;
    for cpu_id in cpus {
        let tracing = tracing.clone();
        listen_to_cpu(cpu_id, &mut queue, tracing)?;
    }

    signal::ctrl_c().await?;
    info!("Exiting...");
    Ok(())
}
