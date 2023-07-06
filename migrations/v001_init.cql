CREATE KEYSPACE IF NOT EXISTS trading_post
    WITH replication = {'class': 'NetworkTopologyStrategy', 'replication_factor': 3};

CREATE TABLE IF NOT EXISTS trading_post.trade (
    id uuid,
    item_id uuid,
    item_name text,
    bid_price bigint,
    buyout_price bigint,
    created_by uuid,
    created_by_username text,
    created_at timestamp,
    expired_at timestamp,
    PRIMARY KEY (id, item_id)
) WITH caching = {'enabled': 'true'} AND compression = {'sstable_compression': 'LZ4Compressor'};