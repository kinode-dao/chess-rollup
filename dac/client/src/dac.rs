use alloy_primitives::{Address as EthAddress, Signature, U256};
use serde::{Deserialize, Serialize};

pub type BatchId = U256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DacRequests {
    Join(EthAddress),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DacResponse {
    BatchVerification {
        batch_id: BatchId,
        sig: Signature,
        // TODO they probably need to sign a more complex message to avoid replay attacks etc.
        // maybe just make it EIP712 compliant?
    },
}
