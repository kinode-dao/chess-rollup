use serde::{Deserialize, Serialize};
use sp1_core::SP1Stdin;

#[derive(Serialize, Deserialize)]
pub struct ProveRequest {
    pub elf: Vec<u8>,
    pub input: SP1Stdin,
}
