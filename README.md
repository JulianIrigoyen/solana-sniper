# The lips of wisdom are closed, except to the ears of Understanding

** this is a fork of rust-websocket-server

[RTFM](https://solana.com/es/docs/rpc) <3

## Features: 

1. Analyzes holders and token distribution for a given SPL address. Follow this example to basically do anything you want.
2. Track new tokens =>
   2.1 Subscribe to logs mentioning Raydium (more to come) and filter initialize2 events.
   2.2 Grab the signature and fetch the transaction to find the main token address.
   2.3 Fetch new token metadata
   NTH -> Snipe here
3. Track wallets => WIP


## Nice to have:

### Technical Indicators

- You receive alerts when there are some buy/sell signals following professional technical indicators. Recommended for professional users.
(Example: "EMA of SOL/USD 1h cross up 70”)

### Token Stats Performance

- You receive alerts when token(s) changes its non-price parameters such as volume, number of trades, ect. in a certain time frames.
(Example: "SOL Price Change % in 1h is greater than 30%”)

### Trading Events

- You receive alerts when specific actions happened, such as large buys, large sells or any trades by a wallet.
(Example: "Wallet HhfmVzo...NxAFFKbWU2h (Solana) has a trade with value greater than $100k at Jupiter”)

Market Movements

You receive notifications following market events such as new trending tokens or new tokens listed.
(Example: "SOL gets into Top10 Trending list")




### Run

You will need to install [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html) and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).
Pull the repo, cd into it and run: 

1. `cargo build`
2. `cargo run`

This app uses Actix to expose an HTTP server, which you can test by making a request to `http://localhost:8080/api/holders` with the following body: 
```json
{
    "token_mint_addresses":["DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"] 
}
```

##### Endpoints
The list below summarizes the available endpoints through RPC aganst a public or private solana validator node:

1. **getLatestBlockhash**
   Purpose: Fetches the latest block hash along with its validity period. Essential for ensuring transactions are recent and will be accepted by the network.
2. **getProgramAccounts**
   Purpose: Retrieves all accounts owned by a specific program, useful for monitoring smart contracts, especially DeFi protocols, and NFT collections.
3. **getSignaturesForAddress**
   Purpose: Returns the signatures of transactions that involve a specific account. This is crucial for tracking transactions related to specific tokens or wallets, providing insights into market activity.
4. **getTransaction**
   Purpose: Fetches a confirmed transaction by its signature. Vital for analyzing transaction details, including participants, token amounts, and more.
5. **getAccountInfo**
   Purpose: Retrieves information about a specific account, including its current balance and owner program. This can be used to monitor the balances of key accounts, such as token treasuries or large holders.
6. **getTokenAccountBalance**
   Purpose: Returns the token balance of a specific SPL token account. It's useful for tracking the distribution and concentration of tokens among holders.
7. **getTokenAccountsByOwner**
   Purpose: Finds all SPL token accounts owned by a specific account. This is useful for identifying all the tokens held by a particular investor or contract.
8. **getTokenSupply**
   Purpose: Provides the total supply of an SPL token. Monitoring changes in token supply can offer insights into inflationary or deflationary pressures on a token's value.
9. **getSlot**
   Purpose: Retrieves the current slot, which is a measure of time in the Solana blockchain. It's useful for understanding the blockchain's state and the timing of transactions.
10. **getSlotLeader**
    Purpose: Identifies the current slot leader, which is the validator node responsible for producing blocks in the current slot. This can provide insights into network dynamics and validator performance.

## Use Case

### Identifying and Monitoring Top Traders
- Use getSignaturesForAddress and getTransaction to track the activities of known wallets associated with top traders. 
### Token Holder Analysis for Decentralization and Whale Tracking

#### WebSocket Data:

1. [accountSubscribe](https://solana.com/es/docs/rpc/websocket/logssubscribe): Monitor changes to specific accounts in real-time, such as token balances changing.
2. [logsSubscribe](https://solana.com/es/docs/rpc/websocket/logssubscribe): Get real-time streaming of transaction logs, useful for live monitoring of contract interactions.
3. [signatureSubscribe](https://solana.com/es/docs/rpc/websocket/signaturesubscribe): Subscribe to receive a notification when the transaction with the given signature reaches the specified commitment level.
4. [blockSubscribe](https://solana.com/es/docs/rpc/websocket/blocksubscribe): Subscribe to receive notification anytime a new block is confirmed or finalized.
5. [programSubscribe](https://solana.com/es/docs/rpc/websocket/programsubscribe): Subscribe to a program to receive notifications when the lamports or data for an account owned by the given program changes









