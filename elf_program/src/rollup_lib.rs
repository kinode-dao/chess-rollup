use alloy_primitives::{Address as AlloyAddress, Signature, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rollup state must contain:
/// - a list of sequenced transactions (used for proving the computation on-chain)
/// - list of balances (for the gas token)
/// - a list of withdrawals (for use on the provided L1 bridge)
/// - additional state T, which can be anything
#[derive(Serialize, Deserialize)]
pub struct RollupState<T, D> {
    pub sequenced: Vec<WrappedTransaction<D>>,
    pub balances: HashMap<AlloyAddress, U256>,
    pub withdrawals: Vec<(AlloyAddress, U256)>,
    pub state: T,
}

/// This is how transactions must be signed and verified for each rollup
/// The event loop will ingest the public key, signature, and transaction data
/// NOTE: T needs a deterministic way to (de)serialize itself - down to the byte
///     otherwise sig verification will be very irritating. So 0x0A and 0x0a are different
///     depending on your serialization method
/// TODO: need to add:
///     nonce
///     value
///     gas
///     gasPrice
///     gasLimit
///     ...
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WrappedTransaction<T> {
    pub pub_key: AlloyAddress,
    pub sig: Signature,
    pub data: TransactionData<T>,
}

/// All rollups must support a few basic transactions:
/// - depositing tokens from L1 to L2
/// - withdrawing tokens from L2 to L1
/// - transferring the gas token between accounts
/// - an enum T for any additional transaction types you may want to add
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TransactionData<T> {
    BridgeTokens(U256),
    WithdrawTokens(U256),
    Transfer {
        from: AlloyAddress,
        to: AlloyAddress,
        amount: U256,
    },
    Extension(T),
}
