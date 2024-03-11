use alloy_primitives::{Signature, B256, U256};
use alloy_signer::Signer;
use alloy_signer::{SignerSync, Wallet};
use k256::ecdsa::SigningKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use kinode_process_lib::{
    await_message, call_init, println, Address, ProcessId, Request, Response,
};

mod dac;
use dac::*;
mod tx;
use tx::WrappedTransaction;

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DacClientState {
    signer: B256,
    batches: HashMap<BatchId, Vec<WrappedTransaction>>,
}

call_init!(init);
fn init(our: Address) {
    println!("dac: begin");

    // TODO save/load state
    let mut state = DacClientState {
        signer: B256::ZERO,
        batches: HashMap::new(),
    };

    let wallet = if state.signer == B256::ZERO {
        let wallet = Wallet::random();
        state.signer = wallet.to_bytes();
        wallet
    } else {
        Wallet::from_bytes(&state.signer).unwrap()
    };

    let _address = wallet.address();

    loop {
        match handle_message(&our, &mut state, wallet.clone()) {
            Ok(()) => {}
            Err(e) => {
                println!("dac: error: {:?}", e);
            }
        };
    }
}

fn handle_message(
    our: &Address,
    state: &mut DacClientState,
    wallet: Wallet<SigningKey>,
) -> anyhow::Result<()> {
    let message = await_message()?;
    println!("dac: received message");

    if !message.is_request() {
        return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
    }

    let body = message.body();
    let source = message.source();
    let batch: Vec<WrappedTransaction> = serde_json::from_slice(body)?;
    state.batches.insert(U256::from(state.batches.len()), batch);

    // TODO since body will be a ton of data at some point, I need to hash body
    // let hash = alloy_primitives::utils::eip191_hash_message(body); // this is potentially a lot of data...

    let _ = Response::new()
        .body(serde_json::to_vec(&DacResponse::BatchVerification {
            batch_id: U256::from(state.batches.len() - 1),
            sig: wallet.sign_message_sync(body).unwrap(),
        })?)
        .send()?;
    // TODO need a message for reading the data
    Ok(())
}
