use crate::generated::{POSTGRES_FUNCS, POSTGRES_TOP_LEVEL_COMMAND_FUNCS};
use aya::{include_bytes_aligned, programs::UProbe, Bpf};
use aya_log::BpfLogger;
use log::{debug, warn};
use std::error::Error;

pub fn init_bpf() -> Result<Bpf, Box<dyn Error>> {
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

    // Embed the eBPF object file as raw bytes at compile-time and load it at runtime.
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/kernelspace-ebpf-tracer"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/kernelspace-ebpf-tracer"
    ))?;

    // Uncomment if a log statement is added to the eBPF program. This will
    // print the log messages to the console.
    if let Err(e) = BpfLogger::init(&mut bpf) {
        warn!("failed to initialize eBPF logger: {}", e);
    }
    Ok(bpf)
}

/// Attach uprobes to the functions we want to trace in postgres.
pub fn attach_uprobes(bpf: &mut Bpf) -> Result<(), Box<dyn Error>> {
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
