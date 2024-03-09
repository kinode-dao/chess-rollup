#![feature(let_chains)]
use alloy_primitives::{Address as EthAddress, Signature, U256};
use kinode_process_lib::{
    await_message, call_init, get_typed_state, println, set_state, Address, Message, NodeId,
    Request, Response,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

mod tx;
use tx::WrappedTransaction;

type BatchId = U256;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisperserState {
    batches: HashMap<BatchId, Vec<WrappedTransaction>>,
    clients: HashSet<(NodeId, EthAddress)>, // TODO DAC needs to be on-chain
    signatures: HashMap<BatchId, HashMap<NodeId, Signature>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DisperserActions {
    AddClient((NodeId, EthAddress)),
    PostBatch(Vec<WrappedTransaction>),
    PullBatch(BatchId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchVerificationContext {
    who: NodeId,
    batch: BatchId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchVerification {
    batch_id: BatchId,
    sig: Signature,
    // TODO they probably need to sign a more complex message to avoid replay attacks etc.
    // maybe just make it EIP712 compliant?
}

fn save_rollup_state(state: &DisperserState) {
    set_state(&bincode::serialize(&state).unwrap());
    // NOTE this function also needs to include logic for pushing to some DA layer
}

fn load_rollup_state() -> DisperserState {
    match get_typed_state(|bytes| Ok(bincode::deserialize::<DisperserState>(bytes)?)) {
        Some(rs) => rs,
        None => DisperserState {
            batches: HashMap::new(),
            clients: HashSet::new(),
            signatures: HashMap::new(),
        },
    }
}

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

call_init!(initialize);

fn initialize(our: Address) {
    println!("{}: started", our.process());

    let mut state: DisperserState = load_rollup_state();

    main_loop(&our, &mut state);
}

fn main_loop(our: &Address, state: &mut DisperserState) {
    loop {
        match await_message() {
            Err(send_error) => {
                println!("{our}: got network error: {send_error:?}");
                continue;
            }
            Ok(message) => match message {
                Message::Request { body, source, .. } => {
                    match handle_request(&our, &source, &body, state) {
                        Ok(()) => continue,
                        Err(e) => println!("{our}: error handling request: {:?}", e),
                    }
                }
                Message::Response { body, context, .. } => {
                    match handle_response(&our, &context, &body, state) {
                        Ok(()) => continue,
                        Err(e) => println!("{our}: error handling request: {:?}", e),
                    }
                }
            },
        }
    }
}

fn handle_request(
    our: &Address,
    source: &Address,
    body: &[u8],
    state: &mut DisperserState,
) -> anyhow::Result<()> {
    println!("disperser: got request");
    if source.node != our.node {
        println!("disperser: got foreign message, ignoring");
        return Ok(());
    }
    match serde_json::from_slice::<DisperserActions>(body)? {
        DisperserActions::PostBatch(txs) => {
            println!("disperser: got post batch");
            let batch = U256::from(state.batches.len());

            state.batches.insert(batch, txs.clone());

            for (dac_member, _) in state.clients.iter() {
                println!("sending batch to {}", dac_member);
                let _ = Request::new()
                    .target((dac_member, "client", "dac", "goldfinger.os"))
                    .body(serde_json::to_vec(&txs)?)
                    .context(serde_json::to_vec(&BatchVerificationContext {
                        who: dac_member.clone(),
                        batch,
                    })?)
                    .expects_response(5) // TODO should be higher...tune this
                    .send()?;
            }
            Ok(())
        }
        DisperserActions::AddClient(client) => {
            state.clients.insert(client);
            save_rollup_state(state);
            Ok(())
        }
        DisperserActions::PullBatch(batch_id) => {
            let batch = state.batches.get(&batch_id).unwrap();
            let _ = Response::new()
                .body(serde_json::to_vec(&batch)?)
                .send()
                .unwrap();
            Ok(())
        }
    }
}

fn handle_response(
    _our: &Address,
    _context: &Option<Vec<u8>>,
    body: &[u8],
    _state: &mut DisperserState,
) -> anyhow::Result<()> {
    let verification = serde_json::from_slice::<BatchVerification>(body)?;
    println!("TODO verify signature");
    Ok(())
}
