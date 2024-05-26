#!/bin/bash
set -e

CACHE_HIT=$1

if [[ "$CACHE_HIT" == "true" && -f /tmp/.buildx-cache/docker-image.tar ]]; then
  docker load -i /tmp/.buildx-cache/docker-image.tar
fi

git clone https://github.com/docker-library/postgres.git postgres-docker

cd postgres-docker/16/bookworm

# Replace 'amd64 | arm64 | ppc64el | s390x' with 'dummy'
sed -i 's/amd64 | arm64 | ppc64el | s390x/dummy/' Dockerfile

sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; sed -i '\''$a override_dh_strip:\\n\\t# Do nothing, which means dont strip the symbols\\n'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; sed -i '\''s|$(CFLAGS)|-ggdb -Og -g3 -fno-omit-frame-pointer|'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; cd ..; apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/' Dockerfile

# Ensure the build cache directory exists
mkdir -p /tmp/.buildx-cache

# Build the PostgreSQL image with cache
docker buildx build --cache-from type=local,src=/tmp/.buildx-cache --cache-to type=local,dest=/tmp/.buildx-cache,mode=max --tag pg-ferret-postgres-16:latest --load .
