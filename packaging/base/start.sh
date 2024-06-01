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
RUST_LOG=info /usr/local/bin/userspace-collector --runner= > >(prefix_logs "pg-ferret") 2> >(prefix_logs "pg-ferret" >&2) &

# Wait for all background processes to finish
wait -n

# Capture the exit status of the first process to exit
exit $?