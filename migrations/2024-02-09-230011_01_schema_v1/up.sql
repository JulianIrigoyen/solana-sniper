CREATE TABLE polygon.polygon_crypto_level2_book_data (
                id SERIAL PRIMARY KEY,
                event_type VARCHAR NOT NULL,
                pair VARCHAR NOT NULL,
                timestamp BIGINT NOT NULL,
                received_timestamp BIGINT NOT NULL,
                exchange_id BIGINT NOT NULL,
                bid_prices JSONB NOT NULL,
                ask_prices JSONB NOT NULL
        );


CREATE TABLE polygon.polygon_crypto_quote_data (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR NOT NULL,
    pair VARCHAR NOT NULL,
    bid_price DOUBLE PRECISION NOT NULL,
    bid_size DOUBLE PRECISION NOT NULL,
    ask_price DOUBLE PRECISION NOT NULL,
    ask_size DOUBLE PRECISION NOT NULL,
    timestamp BIGINT NOT NULL,
    exchange_id BIGINT NOT NULL,
    received_timestamp BIGINT NOT NULL
);

CREATE TABLE polygon.polygon_crypto_trade_data (
                id SERIAL PRIMARY KEY,
                event_type VARCHAR NOT NULL,
                pair VARCHAR NOT NULL,
                price DOUBLE PRECISION NOT NULL,
                timestamp BIGINT NOT NULL,
                size DOUBLE PRECISION NOT NULL,
                conditions JSONB NOT NULL,
                trade_id VARCHAR,
                exchange_id BIGINT NOT NULL,
                received_timestamp BIGINT NOT NULL
);

CREATE TABLE polygon.polygon_crypto_aggregate_data (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR NOT NULL,
    pair VARCHAR NOT NULL,
    open DOUBLE PRECISION NOT NULL,
    close DOUBLE PRECISION NOT NULL,
    high DOUBLE PRECISION NOT NULL,
    low DOUBLE PRECISION NOT NULL,
    volume DOUBLE PRECISION NOT NULL,
    timestamp BIGINT NOT NULL,
    end_time BIGINT NOT NULL,
    vw DOUBLE PRECISION NOT NULL,
    avg_trade_size BIGINT NOT NULL
);
