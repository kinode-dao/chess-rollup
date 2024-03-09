use alloy_primitives::{Signature, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use kinode_process_lib::{
    await_message, call_init, println, Address, ProcessId, Request, Response,
};

mod tx;
use tx::WrappedTransaction;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

type BatchId = U256;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DacClientState {
    batches: HashMap<BatchId, Vec<WrappedTransaction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchVerification {
    batch_id: BatchId,
    sig: Signature,
    // TODO they probably need to sign a more complex message to avoid replay attacks etc.
    // maybe just make it EIP712 compliant?
}

call_init!(init);
fn init(our: Address) {
    println!("dac: begin");

    let mut state = DacClientState {
        batches: HashMap::new(),
    };

    loop {
        match handle_message(&our, &mut state) {
            Ok(()) => {}
            Err(e) => {
                println!("dac: error: {:?}", e);
            }
        };
    }
}

fn handle_message(our: &Address, state: &mut DacClientState) -> anyhow::Result<()> {
    let message = await_message()?;
    println!("dac: received message");

    if !message.is_request() {
        return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
    }

    let body = message.body();
    let source = message.source();
    let batch: Vec<WrappedTransaction> = serde_json::from_slice(body)?;
    state.batches.insert(U256::from(state.batches.len()), batch);
    let _ = Response::new()
        .body(serde_json::to_vec(&BatchVerification {
            batch_id: U256::from(state.batches.len() - 1),
            sig: Signature::test_signature(),
        })?)
        .send()?;
    // TODO need a message for reading the data
    Ok(())
}
