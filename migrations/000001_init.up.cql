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
    bought_by uuid,
    bought_by_username text,
    expired_at timestamp,
    is_deleted boolean,
    PRIMARY KEY ((item_id, item_name), id, created_by)
);

CREATE INDEX IF NOT EXISTS index_trade_item_name ON trading_post.trade (item_name);
CREATE INDEX IF NOT EXISTS index_trade_created_by ON trading_post.trade (created_by);
CREATE INDEX IF NOT EXISTS index_trade_created_by_username ON trading_post.trade (created_by_username);