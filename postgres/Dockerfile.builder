FROM moby/buildkit:buildx-stable-1

# Install necessary packages
RUN apk add --no-cache \
    git \
    docker-cli

# Clone the PostgreSQL Docker repository
RUN git clone https://github.com/docker-library/postgres.git /postgres-docker

# Set working directory to the relevant Dockerfile location
WORKDIR /postgres-docker/16/bookworm
#alpine3.19

# Copy the script to build the Docker image
COPY build-postgres.sh /usr/local/bin/build-postgres.sh
RUN chmod +x /usr/local/bin/build-postgres.sh

CMD ["/bin/sh", "/usr/local/bin/build-postgres.sh"]

