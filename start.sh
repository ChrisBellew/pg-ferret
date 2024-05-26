#!/bin/bash
set -e

# Function to decorate logs
prefix_logs() {
    local prefix=$1
    while IFS= read -r line; do
        echo "[$prefix] $line"
    done
}

# Run the original PostgreSQL entrypoint script to start postgres. Run it
# in the background so that the script can continue to run. Redirect the
# output to stdout and stderr so that it can be captured by the Docker logs.
/usr/local/bin/docker-entrypoint.sh postgres > >(prefix_logs "postgres") 2> >(prefix_logs "postgres" >&2) &

# Run the pg-ferret binary in the background. Redirect the output to stdout
# and stderr so that it can be captured by the Docker logs.
RUST_LOG=info cargo xtask run --release --runner= > >(prefix_logs "pg-ferret") 2> >(prefix_logs "pg-ferret" >&2) &

echo "Starting Grafana..."
/usr/sbin/grafana-server --config=/etc/grafana/grafana.ini --homepath=/usr/share/grafana > >(prefix_logs "grafana") 2> >(prefix_logs "grafana" >&2) &

echo "Starting Tempo..."
/usr/bin/tempo --config.file=/etc/tempo/tempo.yaml > >(prefix_logs "tempo") 2> >(prefix_logs "tempo" >&2) &

# sleep 20

# PGPASSWORD=mypassword /usr/lib/postgresql/16/bin/psql -U myuser mydb -c "SELECT COUNT(*) from pg_tablespace"

# sleep 200000

# Wait for all background processes to finish
wait -n

# Capture the exit status of the first process to exit
exit $?
