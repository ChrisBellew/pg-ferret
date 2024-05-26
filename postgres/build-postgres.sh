#!/bin/bash
set -e

# Replace 'amd64 | arm64 | ppc64el | s390x' with 'dummy'
sed -i 's/amd64 | arm64 | ppc64el | s390x/dummy/' Dockerfile

sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; sed -i '\''$a override_dh_strip:\\n\\t# Do nothing, which means dont strip the symbols\\n'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; sed -i '\''s|$(CFLAGS)|-ggdb -Og -g3 -fno-omit-frame-pointer|'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; cd ..; apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/' Dockerfile

/usr/local/bin/init-buildx.sh

# Build the PostgreSQL image with debug symbols
docker buildx build --cache-from type=local,src=/tmp/$CACHE_KEY --cache-to type=local,dest=/tmp/$CACHE_KEY,mode=max -t pg-ferret-postgres-16:latest .
