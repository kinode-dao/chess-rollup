#![feature(let_chains)]
use alloy_primitives::{Address as AlloyAddress, Signature};
use alloy_signer::{LocalWallet, Signer, SignerSync};
use kinode_process_lib::{
    await_message, call_init, get_blob, get_typed_state, http, println, set_state, Address, Message,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize)]
struct RollupState {
    sequenced: Vec<WrappedTransaction>,
    balances: HashMap<String, u64>,
    withdrawals: Vec<(String, u64)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct WrappedTransaction {
    // all these are hex strings, maybe move to alloy types at some point
    pub_key: String,
    sig: String,
    data: String, // hex string
                  // TODO probably need to add nonces, value, gas, gasPrice, gasLimit, ... but whatever
                  // I think we could use eth_sendRawTransaction to just send arbitrary bytes to a sequencer
                  // or at the very least we can use eth_signMessage plus an http request to this process
}

#[derive(Serialize, Deserialize)]
enum TxType {
    BridgeTokens(u64),   // TODO U256
    WithdrawTokens(u64), // TODO U256
    Transfer {
        from: String,
        to: String,
        amount: u64, // TODO U256
    },
    Mint {
        to: String,
        amount: u64, // TODO U256
    },
}

fn save_rollup_state(state: &RollupState) {
    set_state(&bincode::serialize(&state).unwrap());
}

fn load_rollup_state() -> RollupState {
    match get_typed_state(|bytes| Ok(bincode::deserialize::<RollupState>(bytes)?)) {
        Some(rs) => rs,
        None => RollupState {
            sequenced: Vec::new(),
            balances: HashMap::new(),
            withdrawals: Vec::new(),
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
    println!("{}: started", our.package());

    // transactions will come in via http
    http::bind_ws_path("/", true, false).unwrap();

    let mut state: RollupState = load_rollup_state();
    main_loop(&our, &mut state);
}

fn main_loop(our: &Address, state: &mut RollupState) {
    loop {
        match await_message() {
            Err(send_error) => {
                println!("{our}: got network error: {send_error:?}");
                continue;
            }
            Ok(message) => match handle_request(&our, &message, state) {
                Ok(()) => continue,
                Err(e) => println!("{our}: error handling request: {:?}", e),
            },
        }
    }
}

fn handle_request(our: &Address, message: &Message, state: &mut RollupState) -> anyhow::Result<()> {
    // no responses
    if !message.is_request() {
        return Ok(());
    }
    if message.source().node != our.node {
        // this is basically where we need to handle cross rollup messages
        println!("got cross rollup message, implementation is TODO");
        return Ok(());
    } else if message.source().node == our.node
        && message.source().process == "http_server:distro:sys"
    {
        // receive HTTP requests and websocket connection messages from our server
        match serde_json::from_slice::<http::HttpServerRequest>(message.body())? {
            http::HttpServerRequest::Http(ref incoming) => {
                match handle_http_request(our, state, incoming) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        http::send_response(
                            http::StatusCode::SERVICE_UNAVAILABLE,
                            None,
                            "Service Unavailable".to_string().as_bytes().to_vec(),
                        );
                        Err(anyhow::anyhow!(
                            "rollup: error handling http request: {e:?}"
                        ))
                    }
                }
            }
            http::HttpServerRequest::WebSocketOpen { .. } => {
                // TODO i don't think I Need this? maybe for streaming events at some point
                Ok(())
            }
            http::HttpServerRequest::WebSocketClose(_channel_id) => {
                // TODO i don't think I Need this? maybe for streaming events at some point
                Ok(())
            }
            http::HttpServerRequest::WebSocketPush { .. } => {
                // TODO i don't think I Need this? maybe for streaming events at some point
                Ok(())
            }
        }
    } else {
        return Err(anyhow::anyhow!("ignoring request"));
    }
}

/// Handle HTTP requests from our own frontend.
fn handle_http_request(
    our: &Address,
    state: &mut RollupState,
    http_request: &http::IncomingHttpRequest,
) -> anyhow::Result<()> {
    if http_request.bound_path(Some(&our.process.to_string())) != "/games" {
        http::send_response(
            http::StatusCode::NOT_FOUND,
            None,
            "Not Found".to_string().as_bytes().to_vec(),
        );
        return Ok(());
    }
    match http_request.method()?.as_str() {
        // on GET: view balances
        "GET" => Ok(http::send_response(
            http::StatusCode::OK,
            Some(HashMap::from([(
                String::from("Content-Type"),
                String::from("application/json"),
            )])),
            serde_json::to_vec(&state.balances)?,
        )),
        // on POST: new transaction
        // TODO think we may need more RPC methods here
        "POST" => {
            let Some(blob) = get_blob() else {
                return Ok(http::send_response(
                    http::StatusCode::BAD_REQUEST,
                    None,
                    vec![],
                ));
            };
            let blob_json = serde_json::from_slice::<WrappedTransaction>(&blob.bytes)?;

            Ok(())
        }
        // Any other method will be rejected.
        _ => Ok(http::send_response(
            http::StatusCode::METHOD_NOT_ALLOWED,
            None,
            vec![],
        )),
    }
}

// NOTE
// This code needs to be lifted into SP1 prover when it's time to zk prove
// places where we return errors, we don't even sequence.
// TODO this should return a state hash and or a withdrawals hash
fn chain_event_loop(tx: WrappedTransaction, state: &mut RollupState) -> anyhow::Result<()> {
    let decode_tx = tx.clone();
    let pub_key_bytes: [u8; 20] = hex::decode(&decode_tx.pub_key).unwrap().try_into().unwrap();
    let pub_key: AlloyAddress = AlloyAddress::from(pub_key_bytes);
    let signature: Signature = bincode::deserialize(&hex::decode(&decode_tx.sig).unwrap()).unwrap();
    let data_bytes = hex::decode(&decode_tx.data).unwrap();

    if signature.recover_address_from_msg(&data_bytes[..]).unwrap() != pub_key_bytes {
        return Err(anyhow::anyhow!("bad sig"));
    }

    let Ok(tx_type) = bincode::deserialize(&data_bytes) else {
        return Err(anyhow::anyhow!("bad tx type"));
    };

    match tx_type {
        TxType::BridgeTokens(amount) => {
            state.balances.insert(
                tx.pub_key.clone(),
                state.balances.get(&tx.pub_key).unwrap_or(&0) + amount,
            );
            state.sequenced.push(tx);
            Ok(())
        }
        TxType::WithdrawTokens(amount) => {
            state.balances.insert(
                tx.pub_key.clone(),
                state.balances.get(&tx.pub_key).unwrap_or(&0) - amount,
            );
            state.withdrawals.push((tx.pub_key.clone(), amount));
            state.sequenced.push(tx);
            Ok(())
        }
        TxType::Transfer { from, to, amount } => {
            state.balances.insert(
                from.clone(),
                state.balances.get(&from).unwrap_or(&0) - amount,
            );
            state
                .balances
                .insert(to.clone(), state.balances.get(&to).unwrap_or(&0) + amount);
            state.sequenced.push(tx);
            Ok(())
        }
        TxType::Mint { to, amount } => {
            state
                .balances
                .insert(to.clone(), state.balances.get(&to).unwrap_or(&0) + amount);
            state.sequenced.push(tx);
            Ok(())
        }
    }
}
