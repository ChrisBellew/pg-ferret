#!/bin/bash
set -e

# Add CFLAGS and --enable-debug to the Dockerfile
#sed -i '/export LLVM_CONFIG/iexport CFLAGS="-ggdb -Og -g3 -fno-omit-frame-pointer"; \\' Dockerfile
#sed -i '/export DEB_BUILD_OPTIONS/iexport CFLAGS="-ggdb -Og -g3 -fno-omit-frame-pointer"; \\' Dockerfile

#sed -i 's/# configure flags/echo $CFLAGS;/' /usr/share/postgresql-common/server/postgresql.mk;

# Replace 'amd64 | arm64 | ppc64el | s390x' with 'dummy'
sed -i 's/amd64 | arm64 | ppc64el | s390x/dummy/' Dockerfile
#sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; echo "Looking"; cat \/usr\/share\/postgresql-common\/server\/postgresql.mk; ls; exit 1/' Dockerfile

#sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get install -y --no-install-recommends devscripts; apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; echo "Looking"; sed -i '\''s|# configure flags|echo HEREEEEEEEE; echo $(CFLAGS)|'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; cat \/usr\/share\/postgresql-common\/server\/postgresql.mk; ls; debuild -b -uc -us; cd ..; dpkg -i *.deb/' Dockerfile

#-ggdb -Og -g3 -fno-omit-frame-pointer
#CFLAGS="-O0 -g"

# GOOD
# sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get install -y --no-install-recommends devscripts; echo "CHECKPOINT downloading source"; apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; echo "CHECKPOINT replacing CFLAGS"; sed -i '\''s|$(CFLAGS)|-ggdb -Og -g3 -fno-omit-frame-pointer|'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; echo "CHECKPOINT makefile"; cat debian\/rules; sed -i '\''$a override_dh_strip:\\n\\t# Do nothing, which means dont strip the symbols\\n'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk;echo "CHECKPOINT makefile postgres"; cat \/usr\/share\/postgresql-common\/server\/postgresql.mk; echo "CHECKPOINT makefile buildflags"; cat \/usr\/share\/dpkg\/buildflags.mk; echo "CHECKPOINT makefile architecture"; cat \/usr\/share\/dpkg\/architecture.mk; echo "CHECKPOINT makefile pkg-info"; cat \/usr\/share\/dpkg\/pkg-info.mk; echo "CHECKPOINT makefile vendor"; cat \/usr\/share\/dpkg\/vendor.mk; ls; echo "CHECKPOINT debuilding"; export CFLAGS="-ggdb -Og -g3 -fno-omit-frame-pointer"; debuild -b -uc -us; cd ..; echo "CHECKPOINT dpkg"; dpkg -i libpq5_*.deb libpq-dev_*.deb libecpg6_*.deb libecpg-compat3_*.deb libpgtypes3_*.deb postgresql-common_*.deb postgresql-client-common_*.deb postgresql-client-16_*.deb postgresql-16_*.deb postgresql-server-dev-16_*.deb/' Dockerfile

# GOOD
# sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get install -y --no-install-recommends devscripts; echo "CHECKPOINT downloading source"; apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; echo "CHECKPOINT makefile"; cat debian\/rules; sed -i '\''$a override_dh_strip:\\n\\t# Do nothing, which means dont strip the symbols\\n'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; echo "CHECKPOINT debuilding"; debuild -b -uc -us; cd ..; echo "CHECKPOINT dpkg"; dpkg -i libpq5_*.deb libpq-dev_*.deb libecpg6_*.deb libecpg-compat3_*.deb libpgtypes3_*.deb postgresql-common_*.deb postgresql-client-common_*.deb postgresql-client-16_*.deb postgresql-16_*.deb postgresql-server-dev-16_*.deb/' Dockerfile

# GOOD - light symbols
# sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; sed -i '\''$a override_dh_strip:\\n\\t# Do nothing, which means dont strip the symbols\\n'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; cd ..; apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/' Dockerfile

sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; sed -i '\''$a override_dh_strip:\\n\\t# Do nothing, which means dont strip the symbols\\n'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; sed -i '\''s|$(CFLAGS)|-ggdb -Og -g3 -fno-omit-frame-pointer|'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; cd ..; apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/' Dockerfile



# NEXT
# sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get install -y --no-install-recommends devscripts; echo "CHECKPOINT downloading source"; apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; echo "CHECKPOINT replacing CFLAGS"; sed -i '\''s|$(CFLAGS)|-ggdb -Og -g3 -fno-omit-frame-pointer|'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; echo "CHECKPOINT makefile"; cat debian\/rules; sed -i '\''$a \\noverride_dh_strip:\\n\\t# Do nothing, which means dont strip the symbols\\n'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; cd ..; echo "CHECKPOINT rebuilding"; export CFLAGS="-ggdb -Og -g3 -fno-omit-frame-pointer"; apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/' Dockerfile

#sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get install -y --no-install-recommends devscripts; apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-$PG_MAJOR*; echo "Looking"; sed -i '\''s|$(CFLAGS)|-ggdb -Og -g3 -fno-omit-frame-pointer|'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; cat \/usr\/share\/postgresql-common\/server\/postgresql.mk; cd ..; apt-get build-dep "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-$PG_MAJOR*; dpkg-buildpackage -rfakeroot -uc -b; cd ..; ls -ln; dpkg -i libpq5_*.deb libpq-dev_*.deb libecpg6_*.deb libecpg-compat3_*.deb libpgtypes3_*.deb postgresql-common_*.deb postgresql-client-common_*.deb postgresql-client-16_*.deb postgresql-16_*.deb postgresql-server-dev-16_*.deb/' Dockerfile



#sed -i 's/apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/apt-get install -y --no-install-recommends devscripts; apt-get source "postgresql-$PG_MAJOR=$PG_VERSION"; cd postgresql-*; echo "Looking"; sed -i '\''s|$(CFLAGS)|-ggdb -Og -g3 -fno-omit-frame-pointer|'\'' \/usr\/share\/postgresql-common\/server\/postgresql.mk; cat \/usr\/share\/postgresql-common\/server\/postgresql.mk; ls; cd ..; apt-get source --compile "postgresql-$PG_MAJOR=$PG_VERSION"/' Dockerfile


#CFLAGS='$(CFLAGS)'



# sed -i 's/--disable-rpath \\/--enable-debug \\\n\t--disable-rpath \\/' Dockerfile
# Replace FROM alpine:3.19 with FROM jeanblanchard/alpine-glibc:3.19
#sed -i 's/FROM alpine:3.19/FROM jeanblanchard\/alpine-glibc:3.19/' Dockerfile

# Print the modified Dockerfile (for verification purposes)
cat Dockerfile

# Build the PostgreSQL image with debug symbols
docker build -t pg-ferret-postgres-16:latest .

# Indicate successful build
echo "PostgreSQL debug image built successfully"
