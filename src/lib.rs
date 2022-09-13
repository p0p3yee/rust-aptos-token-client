use anyhow::{Context, Result};
use aptos_sdk::{
    rest_client::{Client as ApiClient, PendingTransaction},
    types::{
        LocalAccount,
        account_address::AccountAddress, transaction::Module,
    },
    bcs,
};

mod types;
mod module_client;
use module_client::ModuleClient;
use types::{CollectionsResources, CollectionOptions, TokenProperty, CollectionData, TransactionOptions, RoyaltyPoints, Token, TokenId, TokenDataId, TokenData};

use crate::types::TokenStoreResources;

const fn get_hex_address_three() -> AccountAddress {
    let mut addr = [0u8; AccountAddress::LENGTH];
    addr[AccountAddress::LENGTH - 1] = 3u8;
    AccountAddress::new(addr)
}

#[derive(Clone, Debug)]
pub struct TokenClient<'a> {
    api_client: &'a ApiClient,
    module_client: ModuleClient,
    token_transfer_module_client: ModuleClient,
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
        let token_transfer_module_client = ModuleClient::new(
            chain_id, 
            get_hex_address_three(),
            "token_transfers"
        );
        Ok(Self { 
            api_client, 
            module_client,
            token_transfer_module_client,
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
        royalty_payee: Option<AccountAddress>,
        royalty_points: Option<RoyaltyPoints>,
        property: Option<TokenProperty>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let options = options.unwrap_or_default();
        let property = property.unwrap_or_default();
        let royalty_points = royalty_points.unwrap_or_default();
        let royalty_payee = royalty_payee.unwrap_or(account.address());

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
                bcs::to_bytes(&royalty_points.denominator).unwrap(),
                bcs::to_bytes(&royalty_points.numerator).unwrap(),
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
            let v: CollectionsResources = serde_json::from_str(&resources.data.to_string()).expect("Error on parsing account resources");

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

    pub async fn get_token(
        &self, 
        creator: AccountAddress,
        collection_name: String,
        token_name: String,
        property_version: Option<String>
    ) -> Option<Token> {
        let property_version = property_version.unwrap_or("0".to_string());

        let token_data_id = TokenDataId{
            creator,
            collection: collection_name,
            name: token_name,
        };
        self.get_token_for_account(creator, TokenId {
            token_data_id: token_data_id,
            property_version: property_version,
        }).await
    }

    pub async fn get_token_for_account(
        &self,
        account: AccountAddress,
        token_id: TokenId,
    ) -> Option<Token> {
        if let Ok(resource) = self.api_client.get_account_resource(account, "0x3::token::TokenStore").await {
            if let Some(resource) = resource.into_inner() {
                if let Ok(data) = serde_json::from_str::<TokenStoreResources>(&resource.data.to_string()) {
                    if let Ok(item) = self.api_client.get_table_item(
                        data.tokens.handle,
                        "0x3::token::TokenId",
                        "0x3::token::Token",
                        token_id,
                    ).await {
                        if let Ok(token) = serde_json::from_str::<Token>(&item.into_inner().to_string()) {
                            return Some(token)
                        }
                    }
                }
            }
        }
        return None
    }

    pub async fn get_token_data(
        &self,
        creator: AccountAddress,
        collection_name: String,
        token_name: String,
    ) -> Option<TokenData> {
        if let Ok(resource) = self.api_client.get_account_resource(creator, "0x3::token::Collections").await {
            if let Some(resource) = resource.into_inner() {
                if let Ok(data) = serde_json::from_str::<TokenStoreResources>(&resource.data.to_string()) {
                    if let Ok(item) = self.api_client.get_table_item(
                        data.tokens.handle,
                        "0x3::token::TokenDataId",
                        "0x3::token::TokenData",
                        TokenDataId {
                            creator,
                            collection: collection_name,
                            name: token_name,
                        },
                    ).await {
                        if let Ok(token_data) = serde_json::from_str::<TokenData>(&item.into_inner().to_string()) {
                            return Some(token_data)
                        }
                    }
                }
            }
        }
        return None
    }

    pub async fn offer_token(
        &self,
        from_account: &mut LocalAccount,
        to_account: AccountAddress,
        creator: AccountAddress,
        collection_name: String,
        name: String,
        amount: u64,
        property_version: Option<String>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let property_version = property_version.unwrap_or("0".to_string());
        let options = options.unwrap_or_default();

        let signed_txn = self.token_transfer_module_client.build_signed_transaction(
            from_account,
            "offer_script",
            vec![],
                vec![
                bcs::to_bytes(&to_account).unwrap(),
                bcs::to_bytes(&creator).unwrap(),
                bcs::to_bytes(&collection_name).unwrap(),
                bcs::to_bytes(&name).unwrap(),
                bcs::to_bytes(&property_version).unwrap(),
                bcs::to_bytes(&amount).unwrap(),
            ],
            options);

        Ok(self
            .api_client
            .submit(&signed_txn)
            .await
            .context("Failed to submit offer token transaction")?
            .into_inner()
        )
    }
    
    pub async fn claim_token(
        &self,
        account: &mut LocalAccount,
        sender: AccountAddress,
        creator: AccountAddress,
        collection_name: String,
        name: String,
        property_version: Option<String>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let property_version = property_version.unwrap_or("0".to_string());
        let options = options.unwrap_or_default();

        let signed_txn = self.token_transfer_module_client.build_signed_transaction(
            account,
            "claim_script",
            vec![],
                vec![
                bcs::to_bytes(&sender).unwrap(),
                bcs::to_bytes(&creator).unwrap(),
                bcs::to_bytes(&collection_name).unwrap(),
                bcs::to_bytes(&name).unwrap(),
                bcs::to_bytes(&property_version).unwrap(),
            ],
            options);

        Ok(self
            .api_client
            .submit(&signed_txn)
            .await
            .context("Failed to submit claim token transaction")?
            .into_inner()
        )
    }

    pub async fn cancel_token_offer(
        &self,
        account: &mut LocalAccount,
        receiver: AccountAddress,
        creator: AccountAddress,
        collection_name: String,
        name: String,
        property_version: Option<String>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let property_version = property_version.unwrap_or("0".to_string());
        let options = options.unwrap_or_default();
        let signed_txn = self.token_transfer_module_client.build_signed_transaction(
            account,
            "cancel_offer_script",
            vec![],
                vec![
                bcs::to_bytes(&receiver).unwrap(),
                bcs::to_bytes(&creator).unwrap(),
                bcs::to_bytes(&collection_name).unwrap(),
                bcs::to_bytes(&name).unwrap(),
                bcs::to_bytes(&property_version).unwrap(),
            ],
            options);

        Ok(self
            .api_client
            .submit(&signed_txn)
            .await
            .context("Failed to submit claim token transaction")?
            .into_inner()
        )
    }

    pub async fn direct_transfer_token(
        &self,
        account: &mut LocalAccount,
        receiver: &mut LocalAccount,
        creator: AccountAddress,
        collection_name: String,
        name: String,
        amount: u64,
        property_version: Option<String>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let property_version = property_version.unwrap_or("0".to_string());
        let options = options.unwrap_or_default();

        let mut signers = Vec::<&LocalAccount>::new();
        signers.push(receiver);

        let signed_txn = self.token_transfer_module_client.build_multisigned_transaction(
            account,
            signers,
            "direct_transfer_script",
            vec![],
                vec![
                bcs::to_bytes(&creator).unwrap(),
                bcs::to_bytes(&collection_name).unwrap(),
                bcs::to_bytes(&name).unwrap(),
                bcs::to_bytes(&property_version).unwrap(),
                bcs::to_bytes(&amount).unwrap(),
            ],
            options);

        Ok(self
            .api_client
            .submit(&signed_txn)
            .await
            .context("Failed to submit claim token transaction")?
            .into_inner()
        )
    }
}

