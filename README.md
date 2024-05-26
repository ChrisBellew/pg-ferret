<div align="center">

# PG Ferret

[![CI][build-badge]][build-url]

All-in-one tracing toolkit for Postgres. Batteries included.

</div>

## Features

- Observe traces of your queries inside Postgres
- Correlate Postgres query spans with your application spans
- Low overhead auto-instrumentation with eBPF. Rust in the kernel and userspace. [Obligatory _blazingly fast_]
- Built in trace storage with Grafana Tempo and trace visualisation with Grafana
  - Or bring your own OpenTelemetry backend (Grafana Tempo, Jaeger, Zipkin, Honeycomb, Datadog, etc.)
- Special debug build of Postgres included. Small (currently unmeasured) performance overhead
- Intended for non-production use in debugging slow queries
- Built with 💙 using the power of Rust and [Aya](https://github.com/aya-rs/aya)

## Why

- Postgres has various plugins which provide aggregated metrics into queries and locks. These are very useful, but don't tell the full story of a query path in Postgres.
- Knowing more about the internals of Postgres helps make us better engineers.
- Tracing is the gold standard of observability. PG Ferret aims to bring Postgres tracing to the masses.

## How it works

- eBPF is a special runtime that allows sandboxed programs to run in the kernel. One use of eBPF is to have a kernel program attach to parts of code running in userspace (your programs), allowing enhanced observability without running a special debugger. PG Ferret uses eBPF to attach to Postgres which is also running in userspace.
- The eBPF loader needs the memory address of each function you want to attach to. These are commonly sought from the _debugging symbols_ which are produced when a program is built, and provide a mapping from function name to memory address in the compiled executable. Debugging symbols can be embedded within the executable itself and if so, the eBPF loader has everything it needs to inject the eBPF program into the kernel and it attach it to the relevant functions.
- PG Ferret embeds a simple program into the kernel which attaches to a few dozen key functions inside Postgres.
- In a standard build of Postgres the debugging symbols are partially or fully stripped, which is unhelpful for eBPF, so PG Ferret ships with a special build of Postgres with these symbols intact.

[build-url]: https://github.com/ChrisBellew/pg-ferret/actions/workflows/build.yml
