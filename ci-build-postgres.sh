#!/bin/bash
set -e

# cd postgres
# ./build.sh

#!/bin/bash

# Load the cache key from an environment variable
CACHE_KEY=$1

# Ensure the Docker buildx builder is set up
# docker buildx create --use --name mybuilder

# # Load the cache from the GitHub Actions cache
# docker load -i /tmp/docker_cache_$CACHE_KEY.tar || true

if [ -f /tmp/docker_cache_$CACHE_KEY.tar ]; then
  docker load -i /tmp/docker_cache_$CACHE_KEY.tar
fi

#cd postgres
#./build.sh

git clone https://github.com/docker-library/postgres.git /postgres-docker

cd /postgres-docker/16/bookworm


# Replace 'amd64 | arm64 | ppc64el | s390x' with 'dummy'
sed -i 's/amd64 | arm64 | ppc64el | s390x/dummy/' Dockerfile

sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; sed -i '\''$a override_dh_strip:\\n\\t# Do nothing, which means dont strip the symbols\\n'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; sed -i '\''s|$(CFLAGS)|-ggdb -Og -g3 -fno-omit-frame-pointer|'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; cd ..; apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/' Dockerfile

#/usr/local/bin/init-buildx.sh

#docker buildx create --use --name mybuilder
# docker buildx create --use\
#   exec "$@"

# Build the PostgreSQL image with debug symbols
#docker buildx build --cache-from type=local,src=/tmp/$CACHE_KEY --cache-to type=local,dest=/tmp/$CACHE_KEY,mode=max -t pg-ferret-postgres-16:latest .
docker buildx build --cache-from type=local,src=/tmp/docker_cache_$CACHE_KEY --cache-to type=local,dest=/tmp/docker_cache_$CACHE_KEY,mode=max -t pg-ferret-postgres-16:latest .


# # Set variables
# CONTAINER_NAME="postgres_builder"
# IMAGE_NAME="postgres-builder:latest"

# # # Check if the container is already running
# # if [ "$(docker ps -q -f name=$CONTAINER_NAME)" ]; then
# #     echo "Stopping and removing existing container $CONTAINER_NAME..."
# #     #docker stop $CONTAINER_NAME
# #     docker rm $CONTAINER_NAME --force
# # fi

# # Build the preparation Docker image
# docker build -t $IMAGE_NAME ./postgres -f ./postgres/Dockerfile.builder

# # Run the container with Docker socket
# docker run --name $CONTAINER_NAME --rm -v /var/run/docker.sock:/var/run/docker.sock -v /tmp:/tmp -e CACHE_KEY=$CACHE_KEY $IMAGE_NAME



# Build the Docker image with cache
#docker buildx build --cache-from type=local,src=/tmp/docker_cache_$CACHE_KEY --cache-to type=local,dest=/tmp/docker_cache_$CACHE_KEY,mode=max -t myimage:latest .

# Save the cache to a tar file
docker save -o /tmp/docker_cache_$CACHE_KEY.tar pg-ferret-postgres-16:latest