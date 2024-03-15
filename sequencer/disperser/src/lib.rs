#![feature(let_chains)]
use alloy_primitives::{Address as EthAddress, Signature, U256};
use kinode_process_lib::{
    await_message, call_init, get_typed_state, println, set_state, Address, Message, NodeId,
    Request,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// mod dac;
// use dac::*;
mod tx;
use tx::WrappedTransaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisperserState {
    // batches: HashMap<BatchId, Vec<Transaction>>,
    clients: HashSet<(NodeId, EthAddress)>, // TODO DAC needs to be on-chain
    signatures: HashMap<U256, HashMap<NodeId, Signature>>,
}

fn save_rollup_state(state: &DisperserState) {
    set_state(&bincode::serialize(&state).unwrap());
    // NOTE this function also needs to include logic for pushing to some DA layer
}

fn load_rollup_state() -> DisperserState {
    match get_typed_state(|bytes| Ok(bincode::deserialize::<DisperserState>(bytes)?)) {
        Some(rs) => rs,
        None => DisperserState {
            // batches: HashMap::new(),
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
    _our: &Address,
    _source: &Address,
    _body: &[u8],
    _state: &mut DisperserState,
) -> anyhow::Result<()> {
    println!("disperser: got request");
    Ok(())
    // if source.node != our.node {
    //     println!("disperser: got foreign message, ignoring");
    //     return Ok(());
    // }
    // match serde_json::from_slice::<DisperserActions<ChessTransactions>>(body)? {
    //     DisperserActions::PostBatch(txs) => {
    //         println!("disperser: got post batch");
    //         let batch = U256::from(state.batches.len());

    //         state.batches.insert(batch, txs.clone());

    //         for (dac_member, _) in state.clients.iter() {
    //             println!("sending batch to {}", dac_member);
    //             let _ = Request::new()
    //                 .target((dac_member, "client", "dac", "goldfinger.os"))
    //                 .body(serde_json::to_vec(&txs)?)
    //                 .context(serde_json::to_vec(&BatchVerificationContext {
    //                     who: dac_member.clone(),
    //                     batch,
    //                 })?)
    //                 .expects_response(5) // TODO should be higher...tune this
    //                 .send()?;
    //         }
    //         Ok(())
    //     }
    //     DisperserActions::AddClient(client) => {
    //         match Request::new()
    //             .target((client.clone(), "client", "dac", "goldfinger.os"))
    //             .body(serde_json::to_vec(&DacRequest::JoinDac)?)
    //             .send_and_await_response(5)
    //             .unwrap()
    //             .unwrap()
    //         {
    //             Message::Response { body, .. } => {
    //                 // maybe this is overcomplicated and retarded and I should just add the client's eth address manually
    //                 let eth_address = serde_json::from_slice::<EthAddress>(&body)?;
    //                 println!("added client {} to DAC", client);
    //                 state.clients.insert((client, eth_address));
    //             }
    //             SendError => {
    //                 println!("error adding client {} to DAC", client);
    //             }
    //             _ => {
    //                 println!("error adding client {} to DAC", client);
    //             }
    //         }

    //         save_rollup_state(state);
    //         Ok(())
    //     }
    // }
}

fn handle_response(
    _our: &Address,
    _context: &Option<Vec<u8>>,
    _body: &[u8],
    _state: &mut DisperserState,
) -> anyhow::Result<()> {
    // let _verification = serde_json::from_slice::<BatchVerificationContext>(body)?;
    println!("TODO verify signature");
    Ok(())
}
