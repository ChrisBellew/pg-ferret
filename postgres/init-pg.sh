/usr/local/pgsql/bin/pg_ctl -D /usr/local/pgsql/data -l logfile start && \
/usr/local/pgsql/bin/createdb test && \
/usr/local/pgsql/bin/psql -c "CREATE DATABASE wallets" && \
/usr/local/pgsql/bin/psql -c "CREATE USER walletapi WITH SUPERUSER PASSWORD 'password123'" && \
/usr/local/pgsql/bin/pg_ctl -D /usr/local/pgsql/data stop