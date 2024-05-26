FROM postgres:16-bookworm

RUN apt-get update
RUN apt-get install -y binutils

# Install rust so we can build and run our tracer
# RUN apt-get install -y curl 
# RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
# ENV PATH="/root/.cargo/bin:${PATH}"
# RUN rustup toolchain install nightly-2024-05-12
# RUN rustup default nightly-2024-05-12
# RUN rustup component add rust-src --toolchain nightly-2024-05-12-x86_64-unknown-linux-gnu

# # Install GCC for installing bpf-linker, and install bpf-linker
# # bpf-linker is a rust utility for linking bpf programs.
# RUN apt-get install -y build-essential 
# RUN cargo install bpf-linker

# # Install libelf and git for installing bpftool, and install bpftool.
# # bpftool is a linux utility for loading and managing bpf programs.
# RUN apt-get install -y libelf-dev git
# WORKDIR /app
# RUN git clone --recurse-submodules https://github.com/libbpf/bpftool.git
# WORKDIR /app/bpftool/src
# RUN make install

# Install postgres debug symbols.
# The standard postgres image does not include debug symbols, which are needed
# to resolve the addresses of the symbols in the postgres binary.
RUN apt-get install -y postgresql-16-dbgsym

# Tell BCC where to find the debug symbols for postgres
# ENV LIBBCC_SYMFS=/usr/lib/debug
# ENV DEBUGINFOD_URLS="file:///usr/lib/debug"

#RUN apt-get install -y strace
#RUN apt-get install -y postgresql-16-dbg


# WORKDIR /app
# COPY . /app

# Build the eBPF program
# RUN cargo xtask build-ebpf --release

# # Build the tracer
# RUN cargo build --release

# # Run the tracer when the container starts, which will load the eBPF program
# # and collect and forward events from the eBPF program.
# CMD ["/bin/sh", "-c", "cargo xtask run --release --runner='/bin/sh -c'"]