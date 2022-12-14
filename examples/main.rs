use std::str::FromStr;

use aptos_sdk::{rest_client::{Client, FaucetClient, aptos_api_types::U64}, types::LocalAccount, coin_client::CoinClient};
use once_cell::sync::Lazy;
use rust_aptos_token_client::{types::{TokenId, TokenDataId}, TokenClient};
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
    let mut bob = LocalAccount::generate(&mut rand::rngs::OsRng);

    let collection_name = "Example Collection";
    let token_name = "First NFT !";

    println!("\n=== Address ===");
    println!("Alice: {}", alice.address().to_hex_literal());
    println!("Bob: {}", bob.address().to_hex_literal());

    faucet_client.fund(alice.address(), 1_000_000).await.context("Failed to fund Alice's account")?;
    faucet_client.fund(bob.address(), 1_000_000).await.context("Failed to fund Bob's account")?;

    println!("\n=== Initial Balances ===");
    println!(
        "Alice: {:?}",
        coin_client
            .get_account_balance(&alice.address())
            .await
            .context("Failed to get Alice's account balance")?
    );
    println!(
        "Bob: {:?}",
        coin_client
            .get_account_balance(&bob.address())
            .await
            .context("Failed to get Bob's account balance")?
    );

    println!("\n=== Creating NFT Collection ===");

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

    let data = token_client.get_collection_data(
        alice.address(),
        collection_name.to_string()
    )
    .await.context("Collection not found ?")?;
    
    println!("Collection Data: {:?}", data);

    println!("\n=== Creating NFT Token for Collection: `{}` ===", collection_name);

    let tx_hash = token_client.create_token(
        &mut alice,
        collection_name,
        token_name,
        "First NFT Description",
        1,
        "First NFT URI",
        1,
        None,
        None,
        None,
        None,
    ).await.context("Failed to submit create token tx")?;

    println!("\nSubmitted Create Token TX: {}", tx_hash.hash.to_string());

    rest_client.wait_for_transaction(&tx_hash).await.context("Failed on waiting create token tx")?;

    println!("\n=== NFT Created ===");

    let data = token_client.get_token(
        alice.address(),
        collection_name.to_string(),
        token_name.to_string(),
        None,
    )
    .await?;
    
    println!("NFT Token Data: {:?}", data);

    println!("\n=== Offering Token from Alice to Bob ===");

    let creator_address = alice.address();

    let tx_hash = token_client.offer_token(
        &mut alice,
        bob.address(),
        creator_address,
        collection_name.to_string(),
        token_name.to_string(),
        1,
        None,
        None,
    )
    .await?;

    rest_client.wait_for_transaction(&tx_hash).await.context("Failed on waiting offer token tx")?;

    println!("\n=== Accepting Token ===");

    let tx_hash = token_client.claim_token(
        &mut bob,
        alice.address(),
        creator_address,
        collection_name.to_string(),
        token_name.to_string(),
        None,
        None
    ).await?;

    rest_client.wait_for_transaction(&tx_hash).await.context("Failed on waiting claim token tx")?;

    println!("\n=== Bob Accepted token offered by Alice ===");

    let data = token_client.get_token_for_account(
        bob.address(),
        TokenId {
            token_data_id: TokenDataId {
                creator: creator_address,
                collection: collection_name.to_string(),
                name: token_name.to_string(),
            },
            property_version: U64(0u64),
        }
    ).await?;

    println!("Bob's Token Data: {:?}", data);

    let result = token_client.get_token_for_account(alice.address(), TokenId {
        token_data_id: TokenDataId {
            creator: creator_address,
            collection: collection_name.to_string(),
            name: token_name.to_string(),
        },
        property_version: U64(0u64),
    })
    .await;

    if result.is_err() {
        println!("\n=== Expected Token Not Found Error for getting Alice's Token | Token already transferred to Bob ===")
    } else {
        println!("\n=== Unexpected, Token should be transferred to Bob ===");
    }

    println!("\n=== Bob offer token to Alice ===");

    let offer_count_before = token_client.get_token_offer_count(bob.address()).await.unwrap_or(0u64);

    let tx_hash = token_client.offer_token(
        &mut bob,
        alice.address(),
        creator_address,
        collection_name.to_string(),
        token_name.to_string(),
        1,
        None,
        None
    ).await?;

    rest_client.wait_for_transaction(&tx_hash).await.context("Failed on waiting bob offer token to alice tx")?;

    println!("\n=== Offer sent {} ===", tx_hash.hash.to_string());

    let offer_count_after = token_client.get_token_offer_count(bob.address()).await?;

    assert!(offer_count_after > offer_count_before, "Unexpected, offer count is the same after offering");

    println!("\n=== Bob cancel offer to alice ===");

    let cancel_offer_count_before = token_client.get_cancel_offer_count(bob.address()).await.unwrap_or(0u64);

    let tx_hash = token_client.cancel_token_offer(
        &mut bob,
        alice.address(),
        creator_address,
        collection_name.to_string(),
        token_name.to_string(),
        None,
        None
    ).await?;

    rest_client.wait_for_transaction(&tx_hash).await.context("Failed on waiting bob cancel offer tx")?;

    let cancel_offer_count_after = token_client.get_cancel_offer_count(bob.address()).await?;

    assert!(cancel_offer_count_after > cancel_offer_count_before, "Unexpected, cancel offer count is the same after cancelling offer");

    println!("\n=== Alice try to accpet offer now ===");

    let tx_hash = token_client.claim_token(
        &mut alice, 
        bob.address(), 
        creator_address, 
        collection_name.to_string(), 
        token_name.to_string(),
        None,
    None
    ).await?;

    let result = rest_client.wait_for_transaction(&tx_hash).await;

    if result.is_err() {
        println!("Expected Accept Offer Tx Error | Offer already cancelled.")
    } else {
        println!("Unexpected, Offer should already cancelled");
    }

    Ok(())
}