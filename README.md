# trading-post
An in-game auction microservice for games

### How to setup dev environment
- Run a docker compose command to start up a fresh Cassandra cluster: 
```
docker-compose -f docker-compose.dev.yaml up -d
```

- Enable SASI (experimental) feature by changing Cassandra configuration:
```
docker-compose -f .\docker-compose.dev.yaml exec -ti cassandra-node3 mkdir -p /bitnami/cassandra/conf/
docker-compose -f .\docker-compose.dev.yaml exec -ti cassandra-node3 cp /opt/bitnami/cassandra/conf/cassandra.yaml /bitnami/cassandra/conf/cassandra.yaml
docker-compose -f .\docker-compose.dev.yaml exec -ti cassandra-node3 sed -i '/enable_sasi_indexes: false/c\enable_sasi_indexes: true' /bitnami/cassandra/conf/cassandra.yaml
```

- Restart Cassandra nodes after changes:
```
docker-compose -f .\docker-compose.dev.yaml restart cassandra-node1 cassandra-node2 cassandra-node3
```

- Connect to a container with migrations:
```
docker-compose -f docker-compose.dev.yaml exec -ti migrations sh
```

- Run the `cassandra-migrate` command to apply list of migrations:
```
export DB="cassandra://cassandra-node1:9042/trading_post?protocol=4&username=cassandra&password=cassandra"
migrate -source file://migrations/ -database "$DB" up
```
