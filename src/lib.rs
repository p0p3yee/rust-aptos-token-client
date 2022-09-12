use std::time::{ SystemTime, UNIX_EPOCH};
use anyhow::{Context, Result};
use aptos_sdk::{
    rest_client::{Client as ApiClient, PendingTransaction},
    types::{
        LocalAccount,
        chain_id::ChainId,
        transaction::{TransactionPayload, EntryFunction},
        account_address::AccountAddress,
    },
    transaction_builder::TransactionBuilder,
    bcs,
    move_types::{identifier::Identifier, language_storage::ModuleId}
};

#[derive(Clone, Debug)]
pub struct TokenClient<'a> {
    api_client: &'a ApiClient
}

impl<'a> TokenClient<'a> {
    pub fn new(api_client: &'a ApiClient) -> Self {
        Self { api_client }
    }

    pub async fn create_collection_script(
        &self,
        from_account: &mut LocalAccount,
        name: &str,
        description: &str,
        uri: &str,
        max_supply: u64,
        tx_options: Option<TransactionOptions>,
        options: Option<CollectionOptions>,
    ) -> Result<PendingTransaction> {
        let options = options.unwrap_or_default();
        let tx_options = tx_options.unwrap_or_default();

        let chain_id = self
            .api_client
            .get_index()
            .await
            .context("Failed to get chain ID")?
            .inner()
            .chain_id;

        let transaction_builder = TransactionBuilder::new(
            TransactionPayload::EntryFunction(EntryFunction::new(
                ModuleId::new(
                    AccountAddress::from_hex("0x3").unwrap(),
                    Identifier::new("token").unwrap(),
                ),
                Identifier::new("create_collection_script").unwrap(),
                vec![],
                vec![
                    bcs::to_bytes(name).unwrap(),        // Name
                    bcs::to_bytes(description).unwrap(), // Description
                    bcs::to_bytes(uri).unwrap(),         // Uri
                    bcs::to_bytes(&max_supply).unwrap(), // Total Supply ?
                    bcs::to_bytes(&vec![
                        options.description_mutable, // Description mutable ?
                        options.uri_mutable,         // URI mutable ?
                        options.supply_mutable,      // Maximum amount mutable ?
                    ])
                    .unwrap(),
                ],
            )),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + tx_options.timeout_sec,
            ChainId::new(chain_id),
        )
        .sender(from_account.address())
        .sequence_number(from_account.sequence_number())
        .max_gas_amount(tx_options.max_gas_amount)
        .gas_unit_price(tx_options.gas_unit_price);

        let signed_txn = from_account.sign_with_transaction_builder(transaction_builder);
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

        let chain_id = self
            .api_client
            .get_index()
            .await
            .context("Failed to get chain ID")?
            .inner()
            .chain_id;
        
        let transaction_builder = TransactionBuilder::new(
            TransactionPayload::EntryFunction(EntryFunction::new(
                ModuleId::new(
                    AccountAddress::from_hex("0x3").unwrap(),
                    Identifier::new("token").unwrap(),
                ),
                Identifier::new("create_token_script").unwrap(),
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
            )),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + options.timeout_sec,
            ChainId::new(chain_id),
        )
        .sender(account.address())
        .sequence_number(account.sequence_number())
        .max_gas_amount(options.max_gas_amount)
        .gas_unit_price(options.gas_unit_price);

        let signed_txn = account.sign_with_transaction_builder(transaction_builder);
        Ok(self
            .api_client
            .submit(&signed_txn)
            .await
            .context("Failed to submit create token transaction")?
            .into_inner())
    }

    pub async fn get_collection_data() {}

    pub async fn get_token() {}

    pub async fn get_token_for_account() {}

    pub async fn get_token_data() {}

    pub async fn offer_token() {}
    
    pub async fn claim_token() {}

    pub async fn direct_transfer_token() {}
}

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
