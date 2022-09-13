use std::str::FromStr;

use aptos_sdk::{rest_client::{Client, FaucetClient}, types::LocalAccount, coin_client::CoinClient};
use once_cell::sync::Lazy;
use rust_aptos_token_client::*;
use anyhow::{Context, Result};
use url::Url;

static NODE_URL: Lazy<Url> = Lazy::new(||  Url::from_str("https://fullnode.devnet.aptoslabs.com").unwrap());
static FAUCET_URL: Lazy<Url> = Lazy::new(|| Url::from_str("https://faucet.devnet.aptoslabs.com").unwrap());

#[tokio::main]
async fn main() -> Result<()> {
    let rest_client = Client::new(NODE_URL.clone());
    let faucet_client = FaucetClient::new(FAUCET_URL.clone(), NODE_URL.clone());
    let coin_client = CoinClient::new(&rest_client);
    let token_client = TokenClient::new(&rest_client).await.context("Failed to create token client")?;

    let mut alice = LocalAccount::generate(&mut rand::rngs::OsRng);

    let collection_name = "Example Collection";

    println!("\n=== Address ===");
    println!("Alice: {}", alice.address().to_hex_literal());

    faucet_client.fund(alice.address(), 20_000).await.context("Failed to fund Alice's account")?;

    println!("\n=== Initial Balances ===");
    println!(
        "Alice: {:?}",
        coin_client
            .get_account_balance(&alice.address())
            .await
            .context("Failed to get Alice's account balance")?
    );

    let tx_hash = token_client.create_collection_script(
        &mut alice,
        collection_name,
        "Example description",
        "uri here",
        1_00, 
        None,
        None,
    ).await.context("Failed to submit create collection tx")?;

    println!("\nSubmitted Create Collection TX: {}", tx_hash.hash.to_string());

    rest_client.wait_for_transaction(&tx_hash).await.context("Failed on waiting create collection tx")?;

    println!("\n=== NFT Collection Created ===");

    if let Some(data) = token_client.get_collection_data(alice.address(), collection_name.to_string()).await {
        println!("\nNFT Data: {:?}", data);
    } else {
        println!("\nCollection not found ?");
    }
    

    Ok(())
}