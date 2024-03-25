use alloy_primitives::{keccak256, Address as AlloyAddress, FixedBytes, Signature, U256};
use alloy_sol_types::{sol, SolValue};
use kinode_process_lib::http::IncomingHttpRequest;
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
/// - a map of nonces (for replay protection)
/// - a list of pending withdrawals (not yet included in a batch)
/// - a list of batches (new states that users can withdraw against on L1)
/// - additional state S, which can be anything. In this repo, we use it for storing chess game state
#[derive(Serialize, Deserialize)]
pub struct BaseRollupState<S, T> {
    pub sequenced: Vec<SignedTransaction<T>>,
    pub balances: HashMap<AlloyAddress, U256>,
    pub nonces: HashMap<AlloyAddress, U256>,
    pub withdrawals: Vec<(AlloyAddress, U256)>,
    pub batches: Vec<WithdrawTree>,
    pub l1_block: U256,
    pub state: S,
}

/// a SignedTransaction is just a wrapper around the different operations that your rollup supports
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignedTransaction<T> {
    pub pub_key: AlloyAddress, // TODO: get rid of this - superfluous!
    pub sig: Signature,
    pub tx: Transaction<T>,
}

/// Transaction wraps the actual data that you want to execute.
/// Right now it just contains the data and a nonce, but later it will also need to include gas
///  gasPrice, gasLimit, etc. (TODO)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction<T> {
    pub data: TransactionData<T>,
    pub nonce: U256,
}

/// All rollups must support a few basic transactions:
/// - depositing tokens from L1 to L2
/// - withdrawing tokens from L2 to L1
/// - transferring the gas token between accounts
/// Any remaining "special" transactions can be handled by the extension field.
/// For instance, in this repo we use it for starting chess games, moving pieces, etc.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TransactionData<T> {
    BridgeTokens {
        amount: U256,
        block: U256,
    },
    WithdrawTokens(U256),
    Transfer {
        from: AlloyAddress,
        to: AlloyAddress,
        amount: U256,
    },
    Extension(T),
}

/// The ExecutionEngine is responsible for taking a transaction and applying it to the rollup state
/// In this repo, we impl ExecutionEngine for FullRollupState
/// The goal of this abstraction is to keep the `sequencer` as general as possible, so that it can
/// execute arbitrary rollup code without knowing any specifics about the rollup
/// ```rust
/// impl ExecutionEngine<MyTransactions> for BaseRollupState<MyState, MyTransactions> {
///     fn execute(&mut self, tx: SignedTransaction<MyTransactions>) -> anyhow::Result<()> {
///         // implement your logic here
///     }
/// }
/// ```
pub trait ExecutionEngine<T> {
    fn execute(&mut self, tx: SignedTransaction<T>) -> anyhow::Result<()>;
    fn save(&self) -> anyhow::Result<()>;
    fn load() -> Self;
    fn rpc(&mut self, req: &IncomingHttpRequest) -> anyhow::Result<()>;
}

/// The RpcApi trait implements all the logic for the RPC API
/// There is a good reason to separate this from the ExecutionEngine: the EE should be ignorant of
/// all things related to kinode, http, etc. It should only know how to apply transactions.
// pub trait RpcApi<S, T>
// where
//     S: ExecutionEngine<T>,
// {
//     fn rpc(&mut self, req: &IncomingHttpRequest) -> anyhow::Result<()>;
// }

/// To enable withdrawals, we need to create a Merkle tree of all the pending withdraws.
/// Every time a new batch is made and posted, we generate all the proofs that will let users
/// withdraw against the new state root.
/// NOTE: this is heavily based on the Uniswap MerkleDistributor contract
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WithdrawTree {
    pub root: FixedBytes<32>,
    pub claims: HashMap<AlloyAddress, Claim>,
    pub token_total: U256,
    pub num_drops: usize,
    pub verified: bool,
}

/// The information an adresss needs to withdraw their tokens
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Claim {
    index: usize,
    amount: U256,
    proof: Vec<FixedBytes<32>>,
}

/// To create a new withdraw root, call WithdrawTree::new(pending_withdrawals)
/// The WithdrawTree can then be distributed to users, allowing them to withdraw against the
/// new state root.
impl WithdrawTree {
    pub fn new(withdrawals: Vec<(AlloyAddress, U256)>) -> Self {
        // 1. aggregate any non-unique withdrawals
        let mut unique_withdrawals: HashMap<AlloyAddress, U256> = HashMap::new();
        for (address, amount) in withdrawals {
            *unique_withdrawals.entry(address).or_insert(U256::ZERO) += amount;
        }

        // 2. sort them
        let mut sorted_unique_withdrawals: Vec<(&AlloyAddress, &U256)> =
            unique_withdrawals.iter().collect();
        sorted_unique_withdrawals.sort_by_key(|&(address, _)| address);

        // 3. get the merkle tree layers
        let mut elements = Vec::new();
        for (i, (address, amount)) in sorted_unique_withdrawals.iter().enumerate() {
            elements.push(Self::to_node(U256::from(i), **address, **amount));
        }
        let layers = Self::get_layers(elements.clone());

        // 4. create the claims
        let mut token_total = U256::ZERO;
        let mut claims = HashMap::new();
        for (i, (address, amount)) in sorted_unique_withdrawals.iter().enumerate() {
            token_total += *amount;
            claims.insert(
                **address,
                Claim {
                    index: i,
                    amount: **amount,
                    proof: Self::get_proof(&layers, i),
                },
            );
        }

        Self {
            root: layers.last().unwrap().first().unwrap().clone(),
            num_drops: sorted_unique_withdrawals.len(),
            token_total,
            claims,
            verified: false,
        }
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

    fn get_proof(layers: &Vec<Vec<FixedBytes<32>>>, index: usize) -> Vec<FixedBytes<32>> {
        let mut proof = Vec::new();
        let mut layer = layers[0].clone();
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
