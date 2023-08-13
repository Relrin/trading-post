# trading-post
An in-game auction microservice for games

### How to setup dev environment
- Run a docker compose command to start up a fresh Cassandra cluster: 
```
docker-compose -f docker-compose.dev.yaml up -d
```

- Connect to a container with migrations:
```
docker-compose -f docker-compose.dev.yaml exec -ti migrations sh
```

Run the `cassandra-migrate` command to apply list of migrations:
```
migrate -database cassandra://cassandra-node1:9042/trading_post?protocol=4&username=cassandra&password=cassandra -path ./migrations/ up
```
