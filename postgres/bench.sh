docker run --rm --network host postgres:latest sh -c 'PGPASSWORD=mypass pgbench -h localhost -p 5432 -U myuser -d mydb -i'
docker run --rm --network host postgres:latest sh -c 'PGPASSWORD=mypass pgbench -h localhost -p 5432 -U myuser -d mydb -t 10000 -c 10'
