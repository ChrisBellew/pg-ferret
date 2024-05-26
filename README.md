<div align="center">

# PG Ferret

[![CI][build-badge]][build-url]

All-in-one tracing toolkit for Postgres. Batteries included.

</div>

## Features

- Observe traces of your queries inside Postgres
- Correlate Postgres query spans with your application spans
- Low overhead auto-instrumentation eBPF. Rust in the kernel and userspace
- Built in trace storage with Grafana Tempo and trace visualisation with Grafana
  - Or bring your own OpenTelemetry backend (Grafana Tempo, Jaeger, Zipkin, Honeycomb, Datadog, etc.)
- Special debug build of Postgres included. Small (currently unmeasured) performance overhead
- Intended for non-production use in debugging slow queries
- Built with 💙 using the power of Rust and [Aya](https://github.com/aya-rs/aya)

[build-url]: https://github.com/ChrisBellew/pg-ferret/actions/workflows/build.yml
