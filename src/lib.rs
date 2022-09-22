use std::str::FromStr;

use anyhow::{Context, Result};
use aptos_sdk::{
    rest_client::{Client as ApiClient, PendingTransaction, aptos_api_types::U64},
    types::{
        LocalAccount,
        account_address::AccountAddress,
    },
    bcs, move_types::language_storage::TypeTag,
};

pub mod types;
mod module_client;
use module_client::ModuleClient;
use types::*;

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
            .into_inner()
        )
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

    pub async fn burn_token(
        &self,
        account: &mut LocalAccount,
        creator: AccountAddress,
        collection_name: &str,
        name: &str,
        amount: u64,
        property_version: Option<u64>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let options = options.unwrap_or_default();
        let property_version = property_version.unwrap_or_default();

        let signed_txn = self.module_client.build_signed_transaction(
            account,
            "burn",
            vec![],
                vec![
                bcs::to_bytes(&creator).unwrap(),
                bcs::to_bytes(collection_name).unwrap(),
                bcs::to_bytes(name).unwrap(),
                bcs::to_bytes(&property_version).unwrap(),
                bcs::to_bytes(&amount).unwrap(),
            ],
            options
        );

        Ok(self
            .api_client
            .submit(&signed_txn)
            .await
            .context("Failed to submit burn token transaction")?
            .into_inner()
        )
    }

    pub async fn get_collection_data(&self, account: AccountAddress, collection_name: String) -> Result<CollectionData>{
        let resources = self
            .api_client
            .get_account_resources(account)
            .await
            .context("Account resources not found")?
            .into_inner()
            .into_iter()
            .find(|data| {
                let res = &data.resource_type;
                res.to_string() == "0x3::token::Collections"
            })
            .context("No NFT Collection Found")?;

        let v: CollectionsResources = serde_json::from_str(
            &resources.data.to_string()
        ).context("Error on parsing account resources")?;

        let result = self.api_client.get_table_item(
            v.collection_data.handle,
            "0x1::string::String",
            "0x3::token::CollectionData",
            collection_name
        )
        .await
        .context("Target collection not found in provided account")?;

        let data = serde_json::from_str::<CollectionData>(
            &result.into_inner().to_string()
        ).context("Error on parsing collection data")?;
        
        Ok(data)
    }

    pub async fn get_token(
        &self, 
        creator: AccountAddress,
        collection_name: String,
        token_name: String,
        property_version: Option<u64>
    ) -> Result<Token> {
        let property_version = property_version.unwrap_or(0);

        let token_data_id = TokenDataId{
            creator,
            collection: collection_name,
            name: token_name,
        };

        self.get_token_for_account(creator, TokenId {
            token_data_id: token_data_id,
            property_version: U64(property_version),
        }).await
    }

    pub async fn get_token_for_account(
        &self,
        account: AccountAddress,
        token_id: TokenId,
    ) -> Result<Token> {
        let resource = self
            .api_client
            .get_account_resource(
                account,
                "0x3::token::TokenStore"
            )
            .await
            .context("Error on getting account resource")?
            .into_inner()
            .context("No Token Found")?;

        let data = serde_json::from_str::<TokenStoreResources>(
            &resource.data.to_string()
        ).context("Error on parsing token store resources")?;

        let item = self.api_client.get_table_item(
            data.tokens.handle,
            "0x3::token::TokenId",
            "0x3::token::Token",
            token_id,
        )
        .await
        .context("Target Token ID not found in provided address")?;

        let token = serde_json::from_str::<Token>(
            &item.into_inner().to_string()
        ).context("Error on parsing token")?;

        Ok(token)
    }

    pub async fn get_token_data(
        &self,
        creator: AccountAddress,
        collection_name: String,
        token_name: String,
    ) -> Result<TokenData> {
        let resource = self
            .api_client
            .get_account_resource(
                creator,
                "0x3::token::Collections"
            )
            .await
            .context("Error on getting accoutn resource")?
            .into_inner()
            .context("No NFT Collection found")?;

        let data = serde_json::from_str::<TokenDataStoreResources>(
            &resource.data.to_string()
        ).context("Error on parsing token data store resources")?;

        let item = self.api_client.get_table_item(
            data.token_data.handle,
            "0x3::token::TokenDataId",
            "0x3::token::TokenData",
            TokenDataId {
                creator,
                collection: collection_name,
                name: token_name,
            },
        )
        .await
        .context("Target token not found")?;

        let token_data = serde_json::from_str::<TokenData>(
            &item.into_inner().to_string()
        ).context("Error on parsing token data")?;
        
        Ok(token_data)
    }

    pub async fn offer_token(
        &self,
        from_account: &mut LocalAccount,
        to_account: AccountAddress,
        creator: AccountAddress,
        collection_name: String,
        name: String,
        amount: u64,
        property_version: Option<u64>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let property_version = property_version.unwrap_or(0);
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
        property_version: Option<u64>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let property_version = property_version.unwrap_or(0);
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
        property_version: Option<u64>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let property_version = property_version.unwrap_or(0);
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
            .context("Failed to submit cancel token offer transaction")?
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
        property_version: Option<u64>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let property_version = property_version.unwrap_or_default();
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
            .context("Failed to submit direct transfer token transaction")?
            .into_inner()
        )
    }

    pub async fn list_token_for_swap(
        &self,
        account: &mut LocalAccount,
        creator: AccountAddress,
        collection: String,
        name: String,
        amount: u64,
        min_coin_per_token: u64,
        locked_until_secs: u64,
        property_version: Option<u64>,
        options: Option<TransactionOptions>,
    ) -> Result<PendingTransaction> {
        let property_version = property_version.unwrap_or_default();
        let options = options.unwrap_or_default();

        let signed_txn = self.token_transfer_module_client.build_signed_transaction(
            account,
            "list_token_for_swap",
            vec![
                TypeTag::from_str(&options.coin_type).unwrap()
            ],
            vec![
                bcs::to_bytes(&creator).unwrap(),
                bcs::to_bytes(&collection).unwrap(),
                bcs::to_bytes(&name).unwrap(),
                bcs::to_bytes(&property_version).unwrap(),
                bcs::to_bytes(&amount).unwrap(),
                bcs::to_bytes(&min_coin_per_token).unwrap(),
                bcs::to_bytes(&locked_until_secs).unwrap(),
            ],
            options);

        Ok(self
            .api_client
            .submit(&signed_txn)
            .await
            .context("Failed to submit list token for swap transaction")?
            .into_inner()
        )
    }

    pub async fn get_pending_claims_resources_for_account(
        &self,
        account: AccountAddress,
    ) -> Result<PendingClaimsResources> {
        let resource = self
            .api_client
            .get_account_resource(
                account,
                "0x3::token_transfers::PendingClaims"
            )
            .await
            .context("Error on getting account resource <0x3::token_transfers::PendingClaims>")?
            .into_inner()
            .context("No Pending Claims Found")?;

        let data = serde_json::from_str::<PendingClaimsResources>(
            &resource.data.to_string()
        ).context("Error on parsing pending claims resource")?;

        return Ok(data);
    }

    pub async fn get_token_offer_count(
        &self,
        account: AccountAddress,
    ) -> Result<u64> {
        let data = self.get_pending_claims_resources_for_account(account).await?;
        Ok(data.offer_events.counter.0)
    }

    pub async fn get_token_claim_count(
        &self,
        account: AccountAddress,
    ) -> Result<u64> {
        let data = self.get_pending_claims_resources_for_account(account).await?;
        Ok(data.claim_events.counter.0)
    }

    pub async fn get_cancel_offer_count(
        &self,
        account: AccountAddress,
    ) -> Result<u64> {
        let data = self.get_pending_claims_resources_for_account(account).await?;
        Ok(data.cancel_offer_events.counter.0)
    }

}

