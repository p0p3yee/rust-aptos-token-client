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