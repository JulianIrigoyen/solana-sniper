# solana-sniper

solana-sniper is a Rust tool that watches the Solana chain over a websocket and summarizes what tracked wallets do. It also serves a small HTTP API for token holders, whales, and transactions. The code is a proof of concept from March 2024. It is a fork of an earlier `rust-websocket-server`.

## Stack

- Rust 2021, tokio
- tokio-tungstenite for the Solana websocket, reqwest for JSON-RPC calls
- actix-web for the HTTP API
- diesel and PostgreSQL (the schema is a leftover from the Polygon.io server this started from; the tracker does not write to it yet)
- A Solana RPC provider (the code reads QuickNode URLs from the environment) and the Moralis and Birdeye HTTP APIs

## How it works

1. `main.rs` opens a websocket to the Solana RPC and sends a `logsSubscribe` request for one tracked wallet.
2. `subscriber/consume_stream.rs` parses each message into a `SolanaEventTypes` value and puts it on a bounded channel.
3. A task reads the channel. For each successful log it calls `getTransaction` over HTTP with the signature.
4. `decoder/tx_decoder.rs` parses the instructions (`TransferChecked` only) and the pre and post token balances. It prints a transaction summary.
5. `server/http_server.rs` serves a JSON API on `127.0.0.1:8080` under `/api`: `holders`, `whales`, `transactions`, `accounts`, `signatures_for_address`, and Birdeye token prices.

The `holders` endpoint takes a list of token mints and returns the largest holders and their share of supply. The `whales` endpoint writes CSV reports. Example reports from March 2024 are in `data/`.

## Data files

`public/birdeye/amm_providers.json` holds public AMM program addresses from Birdeye. They are not secrets. Secret scanners flag them because they look like keys. The wallet addresses in `main.rs` and `data/` are public chain addresses.

## Run it

You need Rust, PostgreSQL, and the diesel CLI.

1. Create a `.env` file with `DATABASE_URL`, `WS_SERVER_HOST`, `WS_SERVER_PORT`, `PRIVATE_SOLANA_QUICKNODE_WS`, and `PRIVATE_SOLANA_QUICKNODE_HTTP`. Some endpoints also need Moralis and Birdeye keys.
2. Run `./run-migrations.sh`. Edit the user and the password in the script first. The committed password is `REDACTED`.
3. Run `cargo run`.
4. Test the API:

```
curl -X POST http://localhost:8080/api/holders \
  -H 'content-type: application/json' \
  -d '{"token_mint_addresses":["DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"]}'
```

## Status

Proof of concept, 5 to 13 March 2024. 30 commits. Not maintained. The wallet to track is hardcoded in `main.rs`. Telegram alerts and the GPT summaries listed in the roadmap are not built. This is not a trading bot: it never places orders.
