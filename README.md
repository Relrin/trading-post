# trading-post
An in-game auction microservice for games

### Setup migrations locally
```
cassandra-migrate --config-file=migrations/config.yaml --profile=dev --user=cassandra --password=cassandra --hosts scylla-node1 status
```
