use std::time::{ SystemTime, UNIX_EPOCH};
use aptos_sdk::{
    types::{
        LocalAccount,
        chain_id::ChainId,
        transaction::{TransactionPayload, EntryFunction, SignedTransaction},
        account_address::AccountAddress,
    },
    transaction_builder::TransactionBuilder,
    move_types::{identifier::Identifier, language_storage::{ModuleId, TypeTag}}
};

use crate::types::TransactionOptions;

#[derive(Clone, Debug)]
pub struct ModuleClient {
    chain_id: ChainId,
    module: ModuleId,
}

impl ModuleClient {
    pub fn new(
        chain_id: u8,
        module_address: AccountAddress,
        module_name: &str
    ) -> Self {
        Self {
            chain_id: ChainId::new(chain_id),
            module: ModuleId::new(
                module_address,
                Identifier::new(module_name).unwrap(),
            )
        }
    }

    pub fn build_signed_transaction(
        &self,
        account: &mut LocalAccount, 
        function_name: &str,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        tx_opts: TransactionOptions
    ) -> SignedTransaction {
        let transaction_builder = TransactionBuilder::new(
            TransactionPayload::EntryFunction(EntryFunction::new(
                self.module.clone(),
                Identifier::new(function_name).unwrap(),
                ty_args,
                args,
            )),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + tx_opts.timeout_sec,
            self.chain_id,
        )
        .sender(account.address())
        .sequence_number(account.sequence_number())
        .max_gas_amount(tx_opts.max_gas_amount)
        .gas_unit_price(tx_opts.gas_unit_price);

        account.sign_with_transaction_builder(transaction_builder)
    }

    pub fn build_multisigned_transaction(
        &self,
        account: &mut LocalAccount,
        other_accounts: Vec<&LocalAccount>,
        function_name: &str,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        tx_opts: TransactionOptions
    ) -> SignedTransaction {
        
        let transaction_builder = TransactionBuilder::new(
            TransactionPayload::EntryFunction(EntryFunction::new(
                self.module.clone(),
                Identifier::new(function_name).unwrap(),
                ty_args,
                args,
            )),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + tx_opts.timeout_sec,
            self.chain_id,
        )
        .sender(account.address())
        .sequence_number(account.sequence_number())
        .max_gas_amount(tx_opts.max_gas_amount)
        .gas_unit_price(tx_opts.gas_unit_price);

        account.sign_multi_agent_with_transaction_builder(other_accounts, transaction_builder)
    }
}

