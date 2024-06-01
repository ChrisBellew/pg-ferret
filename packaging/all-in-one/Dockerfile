FROM cbellew/pg-ferret:latest

WORKDIR /app

RUN ARCH=$(uname -m) && \
  if [ "$ARCH" = "x86_64" ]; then \
    TEMPO_ARCH="amd64"; \
  elif [ "$ARCH" = "aarch64" ]; then \
    TEMPO_ARCH="arm64"; \
  else \
    echo "Unsupported architecture: $ARCH"; exit 1; \
  fi && \
  curl -L -o tempo_2.0.0_linux_$TEMPO_ARCH.deb https://github.com/grafana/tempo/releases/download/v2.0.0/tempo_2.0.0_linux_$TEMPO_ARCH.deb

# Install Tempo and Grafana
RUN apt-get update \
  && apt-get install -y software-properties-common curl \
  && add-apt-repository "deb https://packages.grafana.com/oss/deb stable main" \
  && curl -s https://packages.grafana.com/gpg.key | apt-key add - \
  && apt-get update \
  && apt-get install -y grafana \
  && dpkg -i tempo_2.0.0_linux_*.deb \
  && rm tempo_2.0.0_linux_*.deb \
  && apt-get install -y grafana \
  && apt-get remove -y software-properties-common curl \ 
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

COPY ./packaging/all-in-one/grafana.ini /etc/grafana/grafana.ini
COPY ./packaging/all-in-one/tempo.yaml /etc/tempo/tempo.yaml
COPY ./packaging/all-in-one/tempo-datasource.yaml /etc/grafana/provisioning/datasources/tempo-datasource.yaml

ENV GF_AUTH_DISABLE_LOGIN_FORM=true
ENV GF_AUTH_ANONYMOUS_ENABLED=true
ENV GF_AUTH_ANONYMOUS_ORG_ROLE=Admin
ENV GF_SECURITY_DISABLE_INITIAL_ADMIN_CREATION=true

COPY /packaging/all-in-one/start.sh /usr/local/bin/start.sh
RUN chmod +x /usr/local/bin/start.sh

EXPOSE 5432 3000

ENTRYPOINT ["/usr/local/bin/start.sh"]
