FROM cbellew/pg-ferret-postgres-16:latest as builder

WORKDIR /app

# Install dependencies and Rust
RUN apt-get update \
    && apt-get install -y wget curl git libelf-dev build-essential software-properties-common \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y \
    && export PATH="/root/.cargo/bin:${PATH}" \
    && rustup toolchain install nightly-2024-05-18 \
    && rustup default nightly-2024-05-18 \
    && rustup component add rust-src --toolchain nightly-2024-05-18 \
    && echo "deb http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-18 main" | tee /etc/apt/sources.list.d/llvm-toolchain-bookworm-18.list \
    && echo "deb-src http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-18 main" | tee -a /etc/apt/sources.list.d/llvm-toolchain-bookworm-18.list \
    && wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add - \
    && apt-get update && apt-get install -y systemtap-sdt-dev libreadline-dev zlib1g-dev libsystemd-dev libicu-dev llvm-18-dev libclang-18-dev libpolly-18-dev libzstd-dev \
    && ln -s /usr/lib/llvm-18/bin/llvm-config /usr/bin/llvm-config \
    && llvm-config --version \
    && cargo install bpf-linker --no-default-features \
    && git clone --recurse-submodules https://github.com/libbpf/bpftool.git \
    && cd bpftool/src && make install && cd /app \
    && rm -rf /var/lib/apt/lists/* /app/bpftool \
    && apt-get clean

# Copy source code and build
COPY ./apps /app
RUN /root/.cargo/bin/cargo xtask build-ebpf --release \
    && /root/.cargo/bin/cargo build --release
    
# Final stage
FROM cbellew/pg-ferret-postgres-16:latest

WORKDIR /app
COPY --from=builder /app/target/release/userspace-collector /usr/local/bin/userspace-collector

COPY /packaging/slim/start.sh /usr/local/bin/start.sh
RUN chmod +x /usr/local/bin/start.sh

EXPOSE 5432

ENTRYPOINT ["/usr/local/bin/start.sh"]
