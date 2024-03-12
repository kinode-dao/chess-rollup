use alloy_primitives::B256;
use alloy_signer::Signer;
use alloy_signer::{SignerSync, Wallet};
use k256::ecdsa::SigningKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use kinode_process_lib::{await_message, call_init, println, Address, Response};

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

    match serde_json::from_slice::<DacRequest>(body)? {
        DacRequest::JoinDac => {
            println!("dac: got post batch");
            let _ = Response::new()
                .body(serde_json::to_vec(&DacResponse::JoinDac(wallet.address()))?)
                .send()
                .unwrap();
        }
        DacRequest::VerifyBatch { batch_id, txs } => {
            state.batches.insert(batch_id, txs.clone());
            // TODO since body will be a ton of data at some point, I need to hash body
            // let hash = alloy_primitives::utils::eip191_hash_message(body); // this is potentially a lot of data...
            let _ = Response::new()
                .body(serde_json::to_vec(&DacResponse::BatchVerification {
                    batch_id,
                    sig: wallet
                        .sign_message_sync(&serde_json::to_vec(&txs)?)
                        .unwrap(),
                })?)
                .send()
                .unwrap();
        }
        DacRequest::ReadBatch(batch_id) => {
            let _ = Response::new()
                .body(serde_json::to_vec(
                    state.batches.get(&batch_id).unwrap_or(&vec![]),
                )?)
                .send()
                .unwrap();
        }
    }

    Ok(())
}
