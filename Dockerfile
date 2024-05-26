FROM cbellew/pg-ferret-postgres-16:latest as builder

WORKDIR /app

# Install dependencies and Rust
RUN apt-get update && \
    apt-get install -y curl git libelf-dev build-essential software-properties-common \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y \
    && export PATH="/root/.cargo/bin:${PATH}" \
    && rustup toolchain install nightly-2024-05-18 \
    && rustup default nightly-2024-05-18 \
    && rustup component add rust-src --toolchain nightly-2024-05-18 \
    && cargo install bpf-linker \
    && git clone --recurse-submodules https://github.com/libbpf/bpftool.git \
    && cd bpftool/src && make install && cd /app \
    && rm -rf /var/lib/apt/lists/* /app/bpftool \
    && apt-get clean

# Copy source code and build
COPY /apps /app
RUN /root/.cargo/bin/cargo xtask build-ebpf --release \
    && /root/.cargo/bin/cargo build --release

# Final stage
FROM cbellew/pg-ferret-postgres-16:latest

WORKDIR /app
COPY --from=builder /app/target/release/userspace-collector /usr/local/bin/userspace-collector

# Install Tempo and Grafana
RUN ARCH=$(uname -m) && \
  if [ "$ARCH" = "x86_64" ]; then \
    TEMPO_ARCH="amd64"; \
  elif [ "$ARCH" = "aarch64" ]; then \
    TEMPO_ARCH="arm64"; \
  else \
    echo "Unsupported architecture: $ARCH"; exit 1; \
  fi && \
  curl -L -o tempo_2.0.0_linux_$TEMPO_ARCH.deb https://github.com/grafana/tempo/releases/download/v2.0.0/tempo_2.0.0_linux_$TEMPO_ARCH.deb && \
  dpkg -i tempo_2.0.0_linux_$TEMPO_ARCH.deb && \
  rm tempo_2.0.0_linux_$TEMPO_ARCH.deb && \
  apt-get install -y grafana && \
  rm -rf /var/lib/apt/lists/*

COPY start.sh /usr/local/bin/start.sh
RUN chmod +x /usr/local/bin/start.sh

COPY grafana.ini /etc/grafana/grafana.ini
COPY tempo.yaml /etc/tempo/tempo.yaml
COPY tempo-datasource.yaml /etc/grafana/provisioning/datasources/tempo-datasource.yaml

ENV GF_AUTH_DISABLE_LOGIN_FORM=true
ENV GF_AUTH_ANONYMOUS_ENABLED=true
ENV GF_AUTH_ANONYMOUS_ORG_ROLE=Admin
ENV GF_SECURITY_DISABLE_INITIAL_ADMIN_CREATION=true

EXPOSE 5432 3000

ENTRYPOINT ["/usr/local/bin/start.sh"]
