FROM alpine:3.19

# Install necessary packages
RUN apk add --no-cache \
    git \
    docker-cli

# Install Docker Buildx
RUN mkdir -p ~/.docker/cli-plugins/ \
    && curl -L https://github.com/docker/buildx/releases/latest/download/buildx-v0.10.4.linux-amd64 -o ~/.docker/cli-plugins/docker-buildx \
    && chmod +x ~/.docker/cli-plugins/docker-buildx

# Clone the PostgreSQL Docker repository
RUN git clone https://github.com/docker-library/postgres.git /postgres-docker

# Set working directory to the relevant Dockerfile location
WORKDIR /postgres-docker/16/bookworm
#alpine3.19

# Copy the script to build the Docker image
COPY build-postgres.sh /usr/local/bin/build-postgres.sh
RUN chmod +x /usr/local/bin/build-postgres.sh

# Create a script to initialize buildx
RUN echo '#!/bin/bash\n\
    set -e\n\
    docker buildx create --use\n\
    exec "$@"' > /usr/local/bin/init-buildx.sh \
    && chmod +x /usr/local/bin/init-buildx.sh

CMD ["/bin/sh", "/usr/local/bin/build-postgres.sh"]

