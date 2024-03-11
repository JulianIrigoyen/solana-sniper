# The lips of wisdom are closed, except to the ears of Understanding

** this is a fork of rust-websocket-server


To monitor the market effectively using the Solana JSON RPC API combined with WebSocket data, we focus on endpoints that provide real-time or near-real-time information about transactions, account balances, and the overall state of the blockchain.

[RTFM](https://solana.com/es/docs/rpc) -  Here are some of the most relevant endpoints for market monitoring purposes:

## Features: 

1. Analyzes holders and token distribution for a given SPL address. Follow this example to basically do anything you want.
2. Decodes Transactions (WIP) =>
   2.1 We use a client to fetch unknown program IDLs. Without these we cannot properly decode base64 data in transaction instructions.
3. 

Coming soon: Sniper! Contact me if you want to collab

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

Now that we can quickly get a sense of a new token's distribution (and whales), we can add confidence to our analysis by checking if they are listed in
(since we are using Solana):
- Jupiter
- Raydium
- Orca
- Openbook

 This can easily be confirmed by putting the program ID in the DEX search bar. What is more interesting is to have immediate alerts about new token mints with high liquidity -> this we snipe. 
 How? We need to find transactions that have the InitializeMint instruction. Easy. Or not so easy when you have 1000s and 1000s of TPS! AH i love solana. 

 


###### Journal -- Solana Meme coin Holders
5/03/24 3am
```json


$RETARDIO "6ogzHhzdrQr9Pgv6hZ2MNze7UrzBMAFyBBWUYp1Fhitx"

Initialized Accounts: 9444 -> 9710 (5/03/24 12pm) -> 9880  (18pm) -> 11665 (8/3/2-24)
Holder Accounts: 4227 -> 4219 -> 4243 -> 4695
Holder Ratio: 0.45 -> 0.43 -> 0.43 -> 0.4 


$WIF
Finding holders for [
"EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm",
]
Initialized Accounts: 152290 -> 162359 (5/03/24 18pm)
Holder Accounts: 63722 -> 68563 
Holder Ratio: 0.42 -> 0.42

$BONK
Finding holders for [
"DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
]
Initialized Accounts: 1075109
Holder Accounts: 645636
Holder Ratio: 0.60

$MII
Finding holders for [
"6yaVaoVREVoTt5euoSFpsxLEia1JnzM6fqZj6UWFok1F",
]
Initialized Accounts: 1142 (5/03/24 18pm)
Holder Accounts: 319
Holder Ratio: 0.28

$PIP 
Finding holders for [
"EEfuxw7vmtoqG3EWzJAo6Txb5z1ci5wvMw2wrHwdQSq1",
]
Initialized Accounts: 5089 (5/03/24 20:20 pm)
Holder Accounts: 2863
Holder Ratio: 0.56

$BABY
Finding holders for [
"5hmf8Jt9puwoqiFQTb3vr22732ZTKYRLRw9Vo7tN3rcz",
]
Initialized Accounts: 3550 (5/03/24 20:20 pm)
Holder Accounts: 1981
Holder Ratio: 0.56

```






##### Research
To monitor the market effectively using the Solana JSON RPC API combined with WebSocket data, we focus on endpoints that provide real-time or near-real-time information about transactions, account balances, and the overall state of the blockchain.

Here are some of the most relevant endpoints for market monitoring purposes:

1. getLatestBlockhash
   Purpose: Fetches the latest block hash along with its validity period. Essential for ensuring transactions are recent and will be accepted by the network.
2. getProgramAccounts
   Purpose: Retrieves all accounts owned by a specific program, useful for monitoring smart contracts, especially DeFi protocols, and NFT collections.
3. getSignaturesForAddress
   Purpose: Returns the signatures of transactions that involve a specific account. This is crucial for tracking transactions related to specific tokens or wallets, providing insights into market activity.
4. getTransaction
   Purpose: Fetches a confirmed transaction by its signature. Vital for analyzing transaction details, including participants, token amounts, and more.
5. getAccountInfo
   Purpose: Retrieves information about a specific account, including its current balance and owner program. This can be used to monitor the balances of key accounts, such as token treasuries or large holders.
6. getTokenAccountBalance
   Purpose: Returns the token balance of a specific SPL token account. It's useful for tracking the distribution and concentration of tokens among holders.
7. getTokenAccountsByOwner
   Purpose: Finds all SPL token accounts owned by a specific account. This is useful for identifying all the tokens held by a particular investor or contract.
8. getTokenSupply
   Purpose: Provides the total supply of an SPL token. Monitoring changes in token supply can offer insights into inflationary or deflationary pressures on a token's value.
9. getSlot
   Purpose: Retrieves the current slot, which is a measure of time in the Solana blockchain. It's useful for understanding the blockchain's state and the timing of transactions.
10. getSlotLeader
    Purpose: Identifies the current slot leader, which is the validator node responsible for producing blocks in the current slot. This can provide insights into network dynamics and validator performance.

## Strategy

### Identifying and Monitoring Top Traders
- Use getSignaturesForAddress and getTransaction to track the activities of known wallets associated with top traders. 
    - By setting alerts on transactions from these wallets, you can quickly react to their moves, assuming their actions might signal market movements.

### Token Holder Analysis for Decentralization and Whale Tracking
- Utilize getTokenAccountsByOwner and related RPC calls to analyze the distribution of token holdings.
- A decentralized distribution is generally a healthy indicator, suggesting a broad base of support and less risk of manipulation.
- Concentration in a few accounts (whales) can signify potential price manipulation or that a few players have significant control over the token's market movements.

#### Combining with WebSocket Data:

1. [accountSubscribe](https://solana.com/es/docs/rpc/websocket/logssubscribe): Monitor changes to specific accounts in real-time, such as token balances changing.
2. [logsSubscribe](https://solana.com/es/docs/rpc/websocket/logssubscribe): Get real-time streaming of transaction logs, useful for live monitoring of contract interactions.
3. [signatureSubscribe](https://solana.com/es/docs/rpc/websocket/signaturesubscribe): Subscribe to receive a notification when the transaction with the given signature reaches the specified commitment level.
4. [blockSubscribe](https://solana.com/es/docs/rpc/websocket/blocksubscribe): Subscribe to receive notification anytime a new block is confirmed or finalized.
5. [programSubscribe](https://solana.com/es/docs/rpc/websocket/programsubscribe): Subscribe to a program to receive notifications when the lamports or data for an account owned by the given program changes


#### Analyzing Block Data
- Transaction Flow => crucial for detecting trends, understanding market dynamics, or identifying significant transfers that could affect market conditions or signal specific activities like large trades or transfers related to known wallets.

#### Analyzing Account Data
- This is how we track whales :). But we need to find the whales first.

#### Analyzing Log Data

- Token Management Operations:
  Token Transfers: Direct transfer of tokens between accounts, a fundamental operation in any token ecosystem.
  Minting: Creation of new tokens, indicating issuance of new assets or rewards.
  Account Initialization: Setting up new token accounts, which could signal new participants entering the ecosystem or existing participants expanding their holdings.


- Program and Contract Deployment:
  New Contract Deployments: The initialization of associated token accounts and other setup operations could indicate the deployment of new smart contracts or the provisioning of infrastructure for new applications on Solana.



#### Filtering Token Management Operations
- Token Transfers: Look for transactions that involve the Transfer or TransferChecked instruction of the SPL Token program (TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA).
  These instructions indicate the movement of tokens between accounts.
- Minting: Identify transactions that call the MintTo instruction of the SPL Token program, which is used to mint new tokens to a specific account.
- Account Initialization: Watch for InitializeAccount or InitializeAccount3 instructions, which are used to set up new token accounts. This is often one of the first steps in interacting with tokens on Solana.
- These operations primarily indicate active engagement with tokens on the blockchain.
- While Token Transfers and Minting reflect ongoing economic activity, Account Initialization can be a stronger signal of new users entering the ecosystem, as it involves setting up new token accounts which could belong to new or existing users expanding their holdings.




#### Volume Analysis

- Historically: Sudden spikes or drops in volume can indicate increased interest or disengagement from the market, respectively. 
- Momentarily: An unexpected increase in volume might signal an upcoming price movement, giving you the chance to enter or exit trades ahead of the curve.


#### Liquidity Analysis
Liquidity refers to how easily an asset can be bought or sold in the market without affecting its price. In the context of Solana and DeFi, it refers to the total value of the token/SOL available in the liquidity pool.=
- HIGH LIQUIDITY IS A SIGNAL OF A HEALTHIER MARKET, because the market can sustain more operations with out affecting the price. SO:
    - High liquidity means large quantities of the token can be traded relatively quickly and with minimal price impact. 
    - Low liquidity means that even small trades can lead to significant price changes.

#### FDV (Fully Diluted Valuation) Analysis
Fully Diluted Valuation refers to the valuation of a token if all its future supply were issued and at current market prices.
It's calculated by multiplying the total maximum supply of the token by its current price.


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











