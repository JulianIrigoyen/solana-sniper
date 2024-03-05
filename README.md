# Creating a Rust Websocket Server with Timely Dataflow

🎵 *Currently playing: Rolbac - Mystical says “In Silence you will hear stillness and black hours”* 🎵

As I set up this modest websocket server to get the rust off my Rust, I started remembering why I enjoyed working with it so much. Coming from writing a lot of Scala, Javascript, and .NET code, Rust has a feel to it that is quite hard to describe. It's fun. It feels powerful. The compiler messages are so… containing? More so than any other compile messages I’ve come across in 6 years. Pairing these detailed and concise messages with ChatGPT and, of course, TFM, building this proof of concept was an extremely fun and enlightening experience.

**Motivation:** As my main portfolio project progresses, I am in need of a way to visualize financial asset data in a more insightful way. I had used timely dataflows in the past to build PoCs, so I’m aware of their power to give life to these streams of data in an elegant, really-hard-to-grasp-at-first way.

But bear with me, we’ll go through it and soon you too will be creating dataflows for your streams!

Think of dataflows as extremely efficient pipelines that you can use to transform and process your data. You can imagine boring strings flying into a tube where countless hands, each with a specific intent and purpose, do “stuff” on these chains of characters and on the other side, what you get is valuable insight. 
Dataflows allow us to create pipelines with extendable powers like this easily. In this case, I wanted to be able to consume massive amounts of data about the crypto markets (at the time of writing, BTC just hit 45k again. An alt season is coming and market sentiment analysis tools can actually be game changers in these times). So yes, I’m setting up an alert system for my telegram, partially por el amor al arte, partially because there is true value hidden behind these magical character chains we’ll be subscribing to…

---

**SO… what are we doing?** We will create a websocket server that will connect to the websocket endpoint provided by the Polygon platform. This websocket features different kinds of events that we will subscribe to, process with timely dataflows, and turn into rich, insightful information.

**The core components of this application include:**
- Subscription and deserialization of websocket messages.
- Channeling of parsed data into dataflows.
- Dynamic data filter application.
- Dynamic alert system.

---

## Subscription and Deserialization

If you have not worked with websockets before, you can think of them as hoses of data that allow your application to very easily begin to be flooded with messages. These messages represent events that the server you are connecting to is ready to distribute with its consumers, people like you and me who are thirsty for data.

So, what we do to subscribe is use X crate to send messages to the Polygon server to:
1. Connect
2. Authenticate
3. &4, N - Subscribe to the events we are interested in.

Now, we need to know the shape that the incoming messages have; otherwise, our application will have no clue what to do with all strings. They are just characters! The deserialization step involves transforming strings into structures that our system understands.

---
## Channeling

Messages need to be sent to and from the websocket connections into/out of dataflows. We use crossbeam channels for this

```rust
use crossbeam_channel::{bounded, Sender};

let (sender, receiver) = bounded::<T>(n);

// Continuously read from the channel and process messages
loop {
    match receiver.recv() {
        Ok(message) => {
        // Process the message
        // For example, update dataflows, broadcast to clients, etc.
        },
        Err(_) => {
        // Handle the case where the sender is dropped/disconnected
        break;
        }
    }
}
```

## Dynamic Filters

The main motivation behind this functionality is to allow users, to create the filters they want. As a user, all you need to know is:
- The structure of the data being filtered.
- The type of operation that you want to perform on the data structure to filter it.

With this in mind, we’ll create a filtering structure that will demand knowing the types being filtered and which operations to apply.


There are examples of how you can set up your own filters on (for now) Polygon's data. 
Moving forward, we will expose an API that will allow you to define and save your own filters,
which will then be applied on on demand dataflows.

--- 
## (In progress) Alerts
    Currently working on adding alerts with telegram
---


---
## DB & Diesel

Write the first migration file in the migrations directory generated from running the  ```diesel setup``` command.
This directory is usually placed at the root of your project, along your src directory. 

If the migrations directory doesn't exist, you can create it manually. When you run ```diesel migration generate```,
it will create the migration file in this directory.

[[WIP]]When you add a migration to your Diesel project and rebuild your Docker image,
the migration file will be included in the Docker image. When the Docker container starts up, if it detects that there are pending migrations,
it will automatically apply them to the database.

Timescale is a postgres extension that helps us deal with time-series data. More on this coming soon.

Setup timescale - needs psql server :
```brew tap timescale/tap```
```brew install timescaledb```
```timescaledb-tune --quiet --yes```
```./timescaledb_move.sh```

We will be using Diesel as ORM to interface with our timescaledb. Define a DATABASE_URL in your .env. Remember that you also need to create a database and probably a user for your application, and a schema to keep things tidy. 
Execute this before running  the migrations. Check the samepl init-db.sql in the /migrations folder. Then, to set up diesel:

1. ```cargo install diesel_cli --no-default-features --features postgres```
2. ```diesel setup```
3. ```diesel migration run```


- Dockerized version: WIP

```docker exec -it rust-websocket-server-db-1 psql -U iriuser -d iridb```

```
docker-compose up db &&
docker exec -it rust-websocket-server-db-1 psql -U iriuser -d iridb
```
---



## Strategy

1. Use Depth Data for Market Sentiment Analysis
   Bids and Asks Imbalance: Analyze the volume and number of bids versus asks to gauge market sentiment:
   -> A higher volume of bids might indicate buying pressure, suggesting an upward price movement.
   -> A higher volume of asks might indicate selling pressure, suggesting a downward price movement.

   Support and Resistance Levels: Identify key support and resistance levels based on the concentration of bids and asks.
   These levels can be crucial for setting stop-loss and take-profit points.


2. Incorporate Trade Data for Momentum and Volume Analysis
    * Trade Volume and Price Movement: Monitor trade volumes and their corresponding price changes to identify momentum.
      -> An increase in trade volume accompanied by a significant price movement can signal the strength of the trend.

    * Recent Trades Analysis: Evaluate the most recent trades for sudden changes in volume or price direction.
      -> This can provide early signs of market shifts that can be capitalized on in scalp trading.

3. Apply Rolling Window Data for Oscillator Calculations
    * Relative Strength Index (RSI): Calculate RSI using rolling window data to identify overbought or oversold conditions.
      -> Look for RSI levels below 30 (OVERSOLD) as a buying opportunity and above 70 (OVERBOUGHT) as a selling opportunity.

    * Moving Averages Convergence Divergence (MACD): Use rolling window data to calculate MACD.
      -> Look for signal line crossovers, MACD crossovers, and divergences with price to make trading decisions.

4. Combine Insights
    * Define Entry Points: Enter trades when depth data indicates strong support/resistance levels.
      -> Trade data shows momentum in the desired direction, and oscillators confirm market conditions (e.g., RSI is oversold for buys).

    * Define Exit Points: Set exit points based on resistance levels from depth data.
      -> A decrease in momentum as shown by trade data, and oscillator indications of overbought conditions or weakening trends.

    * Manage Risk: Use depth data to identify strong support and resistance levels for setting stop-loss orders.
      -> Adjust your position size based on the volatility indicated by trade and oscillator data to manage risk effectively.


---

## Oscillators:

1. **Relative Strength Index (RSI)**
    * Purpose: Measures the speed and change of price movements to identify overbought or oversold conditions.
    * Relevance: Ideal for scalping as it can signal potential reversal points by indicating when a crypto asset is overbought (>70) or oversold (<30).
2. **Stochastic Oscillator**
    * Purpose: Compares a specific closing price of a crypto asset to a range of its prices over a certain period to identify momentum and trend reversal.
    * Relevance: Useful in scalping for spotting overbought/oversold conditions and potential price reversals, especially in a volatile market.
3. **Moving Average Convergence Divergence (MACD)**
    * Purpose: Shows the relationship between two moving averages of a cryptocurrency’s price, indicating trend direction, momentum, and potential reversals.
    * Relevance: The MACD histogram's movements around the zero line can provide entry and exit signals for scalpers, with crossovers indicating trade opportunities.
4. **Awesome Oscillator (AO)**
    * Purpose: Measures market momentum to confirm trends or anticipate reversals by comparing the recent market momentum with the general momentum over a wider frame.
    * Relevance: Its zero-line crossovers and histogram peaks can offer scalpers insights into the strength of the market momentum and potential reversal points.
5. **Commodity Channel Index (CCI)**
    * Purpose: Determines the level of a price relative to its historical average, identifying cyclical trends.
    * Relevance: For scalping, values above +100 can indicate a strong uptrend (consider buying opportunities), while values below -100 suggest a strong downtrend (consider selling opportunities).
6. **Bollinger Bands**
    * Purpose: Not an oscillator in the traditional sense but acts as dynamic resistance and support levels based on price volatility.
    * Relevance: Scalpers can use the bands to identify overbought or oversold conditions. Price touching or crossing the bands may indicate potential entry or exit points.