use crate::WrappedTransaction;
use alloy_primitives::{Address as EthAddress, Signature, U256};
use kinode_process_lib::NodeId;
use serde::{Deserialize, Serialize};

pub type BatchId = U256;

// Sent from Disperser (or user) to DAC client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DacRequest {
    JoinDac, // TODO maybe need to give the client more information about the DAC they are joining...
    VerifyBatch {
        batch_id: BatchId,
        txs: Vec<WrappedTransaction>,
    },
    ReadBatch(BatchId),
}

// Sent from the DAC client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DacResponse {
    JoinDac(EthAddress),
    BatchVerification {
        batch_id: BatchId,
        sig: Signature,
        // TODO they probably need to sign a more complex message to avoid replay attacks etc.
        // maybe just make it EIP712 compliant?
    },
}

// Sent to Disperser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisperserActions {
    AddClient(NodeId),
    // probably need some contracts on chain to manage the dac, signatures, etc...it's basically just a multisig
    PostBatch(Vec<WrappedTransaction>),
}

// context for batch verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchVerificationContext {
    pub who: NodeId,
    pub batch: BatchId,
}
