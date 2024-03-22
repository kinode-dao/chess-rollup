use alloy_primitives::{keccak256, Address as AlloyAddress, FixedBytes, Signature, U256};
use alloy_sol_types::{sol, SolValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

sol! {
    struct Node {
        uint256 index;
        address account;
        uint256 amount;
    }
}

/// Rollup state must contain:
/// - a list of sequenced transactions (used for proving the computation on-chain)
/// - list of balances (for the gas token)
/// - a list of withdrawals (for use on the provided L1 bridge)
/// - additional state T, which can be anything
#[derive(Serialize, Deserialize)]
pub struct RollupState<S, T> {
    pub sequenced: Vec<WrappedTransaction<T>>,
    pub balances: HashMap<AlloyAddress, U256>,
    pub withdrawals: Vec<(AlloyAddress, U256)>,
    pub state: S,
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

/// The execution engine is responsible for taking a transaction and applying it to the rollup state
/// ```rust
/// impl ExecutionEngine<MyTransactions> for RollupState<MyState, MyTransactions> {
///     fn execute(&mut self, tx: WrappedTransaction<MyTransactions>) -> anyhow::Result<()> {
///         // implement your logic here
///     }
/// }
/// ```
pub trait ExecutionEngine<T> {
    fn execute(&mut self, tx: WrappedTransaction<T>) -> anyhow::Result<()>;
}

pub struct WithdrawTree {
    elements: Vec<FixedBytes<32>>,
    layers: Vec<Vec<FixedBytes<32>>>,
}

impl WithdrawTree {
    pub fn new(withdrawals: Vec<(AlloyAddress, U256)>) -> Self {
        let mut unique_withdrawals: HashMap<AlloyAddress, U256> = HashMap::new();
        for (address, amount) in withdrawals {
            *unique_withdrawals.entry(address).or_insert(U256::ZERO) += amount;
        }

        let mut elements = Vec::new();
        let mut sorted_unique_withdrawals: Vec<(&AlloyAddress, &U256)> =
            unique_withdrawals.iter().collect();
        sorted_unique_withdrawals.sort_by_key(|&(address, _)| address);

        for (i, (address, amount)) in sorted_unique_withdrawals.iter().enumerate() {
            elements.push(Self::to_node(U256::from(i), **address, **amount));
        }

        let layers = Self::get_layers(elements.clone());
        Self { elements, layers }
    }

    pub fn root(&self) -> FixedBytes<32> {
        self.layers.last().unwrap().first().unwrap().clone()
    }

    fn to_node(index: U256, address: AlloyAddress, amount: U256) -> FixedBytes<32> {
        keccak256(
            &Node {
                index: U256::from(index),
                account: address,
                amount: amount,
            }
            .abi_encode_packed(),
        )
    }

    fn get_layers(elements: Vec<FixedBytes<32>>) -> Vec<Vec<FixedBytes<32>>> {
        let mut layers = Vec::new();
        layers.push(elements);
        while layers.last().unwrap().len() > 1 {
            layers.push(Self::get_next_layer(layers.last().unwrap().to_vec()));
        }
        layers
    }

    fn get_next_layer(elements: Vec<FixedBytes<32>>) -> Vec<FixedBytes<32>> {
        return elements
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut layer, (idx, el)| {
                if idx % 2 == 0 {
                    // Hash the current element with its pair element
                    layer.push(Self::combined_hash(*el, elements[idx + 1]));
                }
                layer
            });
    }

    fn combined_hash(left: FixedBytes<32>, right: FixedBytes<32>) -> FixedBytes<32> {
        if left == [0; 32] {
            return right;
        }
        if right == [0; 32] {
            return left;
        }
        Self::sort_and_concat(left, right)
    }

    fn sort_and_concat(first: FixedBytes<32>, second: FixedBytes<32>) -> FixedBytes<32> {
        if first < second {
            return keccak256(&[first, second].concat());
        }
        keccak256(&[second, first].concat())
    }

    pub fn get_proof(
        &self,
        index: usize,
        account: AlloyAddress,
        amount: U256,
    ) -> Vec<FixedBytes<32>> {
        // TODO this is definitely wrong...it's just going off the index and not on the node hash...which I guess is fine for a first pass
        let node = Self::to_node(U256::from(index), account, amount);
        let mut proof = Vec::new();
        let mut layer = self.layers[0].clone();
        while layer.len() > 1 {
            let mut sibling_idx = index;
            if sibling_idx % 2 == 0 {
                sibling_idx += 1;
            } else {
                sibling_idx -= 1;
            }
            proof.push(layer[sibling_idx]);
            layer = Self::get_next_layer(layer).to_vec();
            sibling_idx /= 2;
        }
        proof
    }
}
