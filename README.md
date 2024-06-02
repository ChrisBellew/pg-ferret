<div align="center">

# PG Ferret

All-in-one tracing toolkit for Postgres. Batteries included.

![CI Status](https://github.com/ChrisBellew/pg-ferret/actions/workflows/build.yml/badge.svg)
[![Docker](https://img.shields.io/badge/Docker-available-blue.svg?style=flat&logo=docker)](https://hub.docker.com/r/cbellew/pg-ferret/tags)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](https://github.com/ChrisBellew/pg-ferret/blob/main/LICENSE)

![](screenshot.png)

</div>

## Features

🔍️ Observe traces of your queries inside Postgres

🔥 Correlate Postgres query spans with your application spans

⚡️ Low overhead auto-instrumentation with eBPF. Rust in the kernel and userspace [Obligatory _blazingly fast_].

🗃️ Built in trace storage with Grafana Tempo and trace visualisation with Grafana. Or bring your own OpenTelemetry backend (Grafana Tempo, Jaeger, Zipkin, Honeycomb, Datadog, etc).

📦 Special debug build of Postgres included. Small (currently unmeasured) performance overhead

🛠️ Intended for non-production use in debugging slow queries

🚀 Built with 💛 using the power of Rust and [Aya](https://github.com/aya-rs/aya)

## Why

- Postgres has various plugins which provide aggregated metrics into queries and locks. These are very useful, but don't tell the full story of a query path in Postgres.

- Understanding PostgreSQL internals helps make us better engineers.

- Tracing is the gold standard of observability. PG Ferret aims to bring Postgres tracing to the masses.

## Usage

### Quick start

To give it a spin, try the all-in-one Docker image. This creates a container with Postgres, PG Ferret, Grafana Tempo and Grafana inside. Use it just like a normal Postgres container and visit Grafana on port 3000 to view traces. You will need to run it in privileged mode for eBPF to work though.

#### 1. Start the all-in-one container

```sh
docker pull cbellew/pg-ferret-all-in-one:latest &&
docker run -it \
  -e POSTGRES_DB=mydb \
  -e POSTGRES_USER=myuser \
  -e POSTGRES_PASSWORD=mypass \
  --privileged -p 5432:5432 -p 3000:3000 \
  cbellew/pg-ferret-all-in-one:latest
```

#### 2. Wait a second and fire a test query

```sh
docker run --rm \
  -e PGPASSWORD=mypass \
  --network=host \
  postgres:16 \
  /usr/lib/postgresql/16/bin/psql -U myuser -h localhost -p 5432 -d mydb -c \
  "SELECT COUNT(*) FROM pg_tablespace"
```

#### 3. [Open Grafana](http://localhost:3000/explore?left=%7B%22datasource%22%3A%22tempo%22%2C%22queries%22%3A%5B%7B%22queryType%22%3A%22traceqlSearch%22%7D%5D%7D) to check out the traces.

### Slim image - bring your own tracing backend

The all-in-one image provides Grafana and Tempo built in, but if you have your own tracing backend you can use the slim PG Ferret image which just has Postgres and PG Ferret built in. Configure the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable and PG Ferret will send traces there in OTLP format.

#### 1. Start the slim image - example for Honeycomb

```sh
docker pull cbellew/pg-ferret-slim:latest &&
docker run -it \
  -e POSTGRES_DB=mydb \
  -e POSTGRES_USER=myuser \
  -e POSTGRES_PASSWORD=mypass \
  -e OTEL_EXPORTER_OTLP_ENDPOINT=https://api.honeycomb.io \
  -e OTEL_EXPORTER_OTLP_HEADERS=x-honeycomb-team=MYHONEYCOMBAPIKEY \
  --privileged -p 5432:5432 \
  cbellew/pg-ferret-slim:latest
```

#### 2. Wait a second and fire a test query

```sh
docker run --rm \
  -e PGPASSWORD=mypass \
  --network=host \
  postgres:16 \
  /usr/lib/postgresql/16/bin/psql -U myuser -h localhost -p 5432 -d mydb -c \
  "SELECT COUNT(*) FROM pg_tablespace"
```

#### 3. Check the traces in Honeycomb

### Examples

Check out the docker compose examples for [Honeycomb](/examples/honeycomb/docker-compose.yml) and [all-in-one](/examples/all-in-one/docker-compose.yml).

### Configuration

| Environment variable          | Description                                                                                                                                    |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | When using the slim image, this configures the endpoint that traces should be sent to. Auth can be provided with `OTEL_EXPORTER_OTLP_HEADERS`. |
| `OTEL_EXPORTER_OTLP_HEADERS`  | Allows custom headers to be send to the OTLP endpoint. e.g. for Honeycomb: `x-honeycomb-team=MYHONEYCOMBAPIKEY`                                |

## How it works

#### eBPF

- eBPF is a special runtime that allows sandboxed programs to run in the Linux kernel without modifying the kernel source code or loading kernel modules.

- It can be used to trace function calls in userspace (your applications) and kernelspace, process and filter network packets, and monitor low level events.

#### Postgres and symbols

- PG Ferret uses eBPF to attach uprobes (userspace probes) into Postgres while it is running, allowing it to trace function calls inside Postgres without modifying the source code.

- To attach a uprobe to a program you need debugging symbols which tell the loader where to find the compiled function in the executable.

- Official Postgres docker images don't ship with debugging symbols - they are stripped after Postgres is built.

- PG Ferret ships with a special debug build of Postgres which includes a richer set of symbols and skips the stripping step, keeping them intact, embedding the `postgres` executable itself.

#### Tracing flow

- PG Ferret starts a collector program that runs in userspace which has an eBPF program embedded inside of it.

- The userspace collector loads the eBPF into the kernel using the eBPF API.

- The kernel loads the eBPF program and attaches it to the uprobes (function calls) in Postgres.

- Whenever an attached function is called in Postgres, the kernel runs the eBPF code before and after the function is executed. This happens on the same CPU core that the Postgres function is running.

- The eBPF program packages the call information into an `event`, as called by PG Ferret, and adds it to an eBPF map which acts as a queue between the eBPF program in kernelspace and the collector in userspace. An event is emitted for each invocation and return of each function.

- The userspace collector polls the map for events, then collects the events and ties them together into spans.

- The userspace collector prepares the call spans into a trace and emits them using the OpenTelemetry OTLP protocol to a tracing collector. In the all-in-one package of PG Ferret this is a built in Grafana Tempo instance, but in the slim image, it's whatever downstream collector you configure.
