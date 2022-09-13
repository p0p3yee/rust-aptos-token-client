use serde::{Deserialize, Serialize};
use aptos_sdk::{types::account_address::AccountAddress, rest_client::aptos_api_types::U64};
pub struct TransactionOptions {
    pub max_gas_amount: u64,

    pub gas_unit_price: u64,

    /// This is the number of seconds from now you're willing to wait for the
    /// transaction to be committed.
    pub timeout_sec: u64,
}

impl Default for TransactionOptions {
    fn default() -> Self {
        Self {
            max_gas_amount: 5_000,
            gas_unit_price: 1,
            timeout_sec: 10,
        }
    }
}

#[derive(Default)]
pub struct CollectionOptions {
    pub description_mutable: bool,
    pub uri_mutable: bool,
    pub supply_mutable: bool,
}

#[derive(Default)]
pub struct TokenProperty {
    pub keys: Vec<String>,
    pub values: Vec<String>,
    pub types: Vec<String>,
}

#[derive(Default)]
pub struct RoyaltyPoints {
    pub denominator: u64,
    pub numerator: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Handle {
    pub handle: AccountAddress,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionsResources {
    pub collection_data: Handle
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenStoreResources {
    pub tokens: Handle
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionMutabilityConfig {
    pub description: bool,
    pub maximum: bool,
    pub uri: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionData {
    pub name: String,
    pub description: String,
    pub uri: String,
    pub supply: U64,
    pub maximum: U64,
    pub mutability_config: CollectionMutabilityConfig,
}


// NFT Token types

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenData {
    pub collection: String,
    pub description: String,
    pub name: String,
    pub maximum: Option<U64>,
    pub supply: U64,
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenDataId {
    pub creator: AccountAddress,
    pub collection: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenId {
    pub token_data_id: TokenDataId,
    pub property_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    pub id: TokenId,
    pub amount: U64,
}