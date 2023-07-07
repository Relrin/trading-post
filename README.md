# trading-post
An in-game auction microservice for games

### How to setup dev environment
- Run a docker compose command to start up a fresh Cassandra cluster: 
```
docker-compose -f docker-compose.dev.yaml up -d
```

- Connect to a container with migrations:
```
docker-compose -f docker-compose.dev.yaml exec -ti migrations bash
```

Run the `cassandra-migrate` command to apply list of migrations:
```
cassandra-migrate --config-file=migrations/config.yaml --profile=dev --user=cassandra --password=cassandra --hosts cassandra-node1 migrate
```
