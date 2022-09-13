use std::str::FromStr;

use anyhow::{Context, Result};
use aptos_sdk::{
    rest_client::{Client as ApiClient, PendingTransaction},
    types::{
        LocalAccount,
        account_address::AccountAddress,
    },
    bcs,
};

mod types;
mod module_client;
use module_client::ModuleClient;
use types::{AccountResources, CollectionOptions, TokenProperty, CollectionData, TransactionOptions};

const fn get_hex_address_three() -> AccountAddress {
    let mut addr = [0u8; AccountAddress::LENGTH];
    addr[AccountAddress::LENGTH - 1] = 3u8;
    AccountAddress::new(addr)
}

#[derive(Clone, Debug)]
pub struct TokenClient<'a> {
    api_client: &'a ApiClient,
    module_client: ModuleClient,
}

impl<'a> TokenClient<'a> {
    pub async fn new(api_client: &'a ApiClient) -> Result<TokenClient<'a>> {
        let chain_id = api_client
            .get_index()
            .await
            .context("Failed to get chain ID")?
            .inner()
            .chain_id;
        let module_client = ModuleClient::new(
            chain_id, 
            get_hex_address_three(),
            "token"
        );
        Ok(Self { 
            api_client, 
            module_client,
        })
    }

    pub async fn create_collection_script(
        &self,
        from_account: &mut LocalAccount,
        name: &str,
        description: &str,
        uri: &str,
        max_supply: u64,
        options: Option<TransactionOptions>,
        collection_options: Option<CollectionOptions>,
    ) -> Result<PendingTransaction> {
        let options = options.unwrap_or_default();
        let collection_options = collection_options.unwrap_or_default();
        
        let signed_txn = self.module_client.build_signed_transaction(
            from_account,
            "create_collection_script",
            vec![],
            vec![
                bcs::to_bytes(name).unwrap(),        // Name
                bcs::to_bytes(description).unwrap(), // Description
                bcs::to_bytes(uri).unwrap(),         // Uri
                bcs::to_bytes(&max_supply).unwrap(), // Total Supply ?
                bcs::to_bytes(&vec![
                    collection_options.description_mutable, // Description mutable ?
                    collection_options.uri_mutable,         // URI mutable ?
                    collection_options.supply_mutable,      // Maximum amount mutable ?
                ])
                .unwrap(),
            ],
            options,
        );

        Ok(self
            .api_client
            .submit(&signed_txn)
            .await
            .context("Failed to submit create collection transaction")?
            .into_inner())
    }

    pub async fn create_token(
        &self,
        account: &mut LocalAccount,
        collection_name: &str,
        name: &str,
        description: &str,
        supply: u64,
        uri: &str,
        max_mint: u64,
        royalty_payee: AccountAddress,
        royalty_points_denominator: u64,
        royalty_points_numerator: u64,
        property: Option<TokenProperty>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let options = options.unwrap_or_default();
        let property = property.unwrap_or_default();

        let signed_txn = self.module_client.build_signed_transaction(
            account,
            "create_token_script",
            vec![],
                vec![
                bcs::to_bytes(collection_name).unwrap(),
                bcs::to_bytes(name).unwrap(),
                bcs::to_bytes(description).unwrap(),
                bcs::to_bytes(&supply).unwrap(),
                bcs::to_bytes(&max_mint).unwrap(),
                bcs::to_bytes(uri).unwrap(),
                bcs::to_bytes(&royalty_payee).unwrap(),
                bcs::to_bytes(&royalty_points_denominator).unwrap(),
                bcs::to_bytes(&royalty_points_numerator).unwrap(),
                bcs::to_bytes(&vec![false, false, false, false, false]).unwrap(),
                bcs::to_bytes(&property.keys).unwrap(),
                bcs::to_bytes(&property.values).unwrap(),
                bcs::to_bytes(&property.types).unwrap(),
            ],
            options);

        Ok(self
            .api_client
            .submit(&signed_txn)
            .await
            .context("Failed to submit create token transaction")?
            .into_inner()
        )
    }

    pub async fn get_collection_data(&self, account: AccountAddress, collection_name: String) -> Option<CollectionData>{
        if let Ok(resources) = self.api_client.get_account_resources(account).await {
            let resources = resources.into_inner().into_iter().find(|data| {
                let res = &data.resource_type;
                res.to_string() == "0x3::token::Collections"
            });
            // No NFT
            if resources.is_none() {
                return None
            }
            let resources = resources.unwrap();
            let v: AccountResources = serde_json::from_str(&resources.data.to_string()).expect("Error on parsing account resources");

            let result = self.api_client.get_table_item(
                v.collection_data.handle,
                "0x1::string::String",
                "0x3::token::CollectionData",
                collection_name
            ).await;
            let result = result.expect("Error on getting table item");
            let data = serde_json::from_str::<CollectionData>(&result.into_inner().to_string());
            if data.is_err() {
                println!("{:?}", data.err());
                return None;
            } else {
                return Some(data.unwrap())
            }
        } else {
            return None
        }
    }

    pub async fn get_token() {}

    pub async fn get_token_for_account() {}

    pub async fn get_token_data() {}

    pub async fn offer_token() {}
    
    pub async fn claim_token() {}

    pub async fn direct_transfer_token() {}
}

