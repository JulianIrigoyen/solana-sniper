//url https://birdeye.so/leaderboard/today?chain=solana
//leaderboard //*[@id="root"]/div[2]/div[2]/div
//example wallet xpath /html/body/div[1]/div[2]/div[2]/div/div[2]/div[2]/div/div/div/div/div/div/div/div[2]/table/tbody/tr[3]/td[2]/div/a

use fantoccini::{Client, ClientBuilder, error::CmdError, Locator};

///Implementation is ok, chromedriver works, but Birdeye is guarded against automated systems
pub async fn scrape_wallet_addresses() -> Result<Vec<String>, CmdError> {
    let client = ClientBuilder::native()
        .connect("http://localhost:9515")
        .await
        .expect("[[SCRAPER]] Failed to connect to chromedriver");

    let mut wallets = Vec::new();

    client.goto("https://birdeye.so/leaderboard/today?chain=solana").await
        .expect("[[SCRAPER]] Failed to navigate to Birdeye top wallets");

    // Ensure you have the correct XPath for the "TODAY" button; this is just an example
    let button = client.find(Locator::XPath("/html/body/div/div[2]/div[2]/div/div[1]/div[2]/div/label[2]/span[2]")).await?;
    button.click().await
        .expect("[[SCRAPER]] Failed to navigate to Birdeye top wallets TODAY's wallets");

    // Adjust sleep time based on how long the page takes to load dynamically loaded content
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // This CSS selector is a placeholder; ensure it matches the actual elements you want to scrape
    let wallet_elements = client.find_all(Locator::Css("div.leaderboard-address")).await
        .expect("[[SCRAPER]] Failed to find leaderboard wallet addresses");

    for wallet_element in wallet_elements {
        if let Ok(address) = wallet_element.text().await {
            println!("Wallet Address: {:?}", &address);
            wallets.push(address);
        } else {
            eprintln!("Failed to extract wallet address from element {:?}", wallet_element)
        }

    }

    client.close().await.expect("[[SCRAPER]] Failed close client");
    Ok(wallets)
}