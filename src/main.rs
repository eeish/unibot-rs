use dotenv::dotenv;
use ethers::{
    prelude::*,
    providers::{Provider, Ws},
};
use eyre::Result;
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let infura_url: String = env::var("INFURA_MAINNET_WS").unwrap();

    let client = Provider::<Ws>::connect(infura_url).await?;
    let client = Arc::new(client);

    let mut stream = client.subscribe_pending_txs().await?;
    while let Some(tx) = stream.next().await {
        let tx = client.get_transaction(tx).await?.unwrap();
        println!(
            "Ts: {:?}, transaction from: {:?} -> {:?}",
            tx.value, tx.from, tx.to
        );
    }
    Ok(())
}
