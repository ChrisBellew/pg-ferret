FROM cbellew/pg-ferret-postgres-16:latest

RUN apt-get update
WORKDIR /app

# Install rust 
#RUN apt-get update
#RUN apt install -y curl git linux-tools-common libelf-dev pkgconf
RUN apt install -y curl
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup toolchain install nightly-2024-05-18
RUN rustup default nightly-2024-05-18
RUN rustup component add rust-src --toolchain nightly-2024-05-18

# Install bpf-linker
RUN apt install -y build-essential
RUN cargo install bpf-linker

# Install bpftool
RUN apt install -y git libelf-dev
RUN git clone --recurse-submodules https://github.com/libbpf/bpftool.git
WORKDIR /app/bpftool/src
RUN make install
WORKDIR /app

RUN apt-get install -y gosu

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
  rm tempo_2.0.0_linux_$TEMPO_ARCH.deb
RUN apt-get install -y software-properties-common \
  && add-apt-repository "deb https://packages.grafana.com/oss/deb stable main" \
  && curl -s https://packages.grafana.com/gpg.key | apt-key add - \
  && apt-get update \
  && apt-get install -y grafana

RUN rm -rf /var/lib/apt/lists/*

COPY /apps /app
RUN cargo xtask build-ebpf --release
RUN cargo build --release

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