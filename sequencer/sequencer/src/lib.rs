#![feature(let_chains)]
use kinode_process_lib::kernel_types::MessageType;
use kinode_process_lib::{
    await_message, call_init, get_blob, get_typed_state, http, println, set_state, Address,
    Message, Request,
};
use serde::{Deserialize, Serialize};
use sp1_core::SP1Stdin;
use std::collections::HashMap;

mod prover_types;
use prover_types::ProveRequest;
mod tx;
use tx::*;

const ELF: &[u8] = include_bytes!("../../../elf_program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AdminActions {
    Prove,
}

fn save_rollup_state(state: &RollupState) {
    set_state(&bincode::serialize(&state).unwrap());
    // NOTE this function also needs to include logic for pushing to some DA layer
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

    http::serve_ui(&our, "ui", true, false, vec!["/"]).unwrap();
    http::bind_http_path("/rpc", true, false).unwrap();
    // transactions will come in via http
    http::bind_ws_path("/", true, false).unwrap();
    http::bind_ext_path("/").unwrap();

    let mut state: RollupState = load_rollup_state();
    let mut connection: Option<u32> = None;

    main_loop(&our, &mut state, &mut connection);
}

fn main_loop(our: &Address, state: &mut RollupState, connection: &mut Option<u32>) {
    loop {
        match await_message() {
            Err(send_error) => {
                println!("{our}: got network error: {send_error:?}");
                continue;
            }
            Ok(message) => match handle_request(&our, &message, state, connection) {
                Ok(()) => continue,
                Err(e) => println!("{our}: error handling request: {:?}", e),
            },
        }
    }
}

fn handle_request(
    our: &Address,
    message: &Message,
    state: &mut RollupState,
    connection: &mut Option<u32>,
) -> anyhow::Result<()> {
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
            http::HttpServerRequest::WebSocketOpen { ref channel_id, .. } => {
                println!("sequencer: got WebSocketOpen, connection established");
                *connection = Some(*channel_id);
                Ok(())
            }
            http::HttpServerRequest::WebSocketClose(ref channel_id) => {
                println!("sequencer: got WebSocketClose");
                if connection.unwrap_or(0) != *channel_id {
                    return Err(anyhow::anyhow!("wrong channel_id"));
                }
                *connection = None;
                Ok(())
            }
            http::HttpServerRequest::WebSocketPush {
                ref channel_id,
                ref message_type,
            } => {
                if connection.unwrap_or(0) != *channel_id {
                    return Err(anyhow::anyhow!("wrong channel_id"));
                }
                println!("sequencer: got WebSocketPush");
                let http::WsMessageType::Binary = message_type else {
                    return Err(anyhow::anyhow!("expected binary message"));
                };

                let Some(blob) = get_blob() else {
                    return Err(anyhow::anyhow!("WebSocketPush Binary had no blob"));
                };

                let kinode_process_lib::http::HttpServerAction::WebSocketExtPushData {
                    id,
                    kinode_message_type,
                    blob,
                } = serde_json::from_slice(&blob.bytes)?
                else {
                    return Err(anyhow::anyhow!("expected WebSocketExtPushData"));
                };
                println!("got proof, {:?}", String::from_utf8(blob));
                // TODO do something with the proof
                Ok(())
            }
        }
    } else if message.source().node == our.node {
        match serde_json::from_slice::<AdminActions>(message.body())? {
            AdminActions::Prove => {
                let Some(channel_id) = connection else {
                    return Err(anyhow::anyhow!("no connection"));
                };

                let mut stdin = SP1Stdin::new();
                stdin.write(&state.sequenced.clone());

                Request::new()
                    .target("our@http_server:distro:sys".parse::<Address>()?)
                    .body(serde_json::to_vec(
                        &http::HttpServerAction::WebSocketExtPushOutgoing {
                            channel_id: *channel_id,
                            message_type: http::WsMessageType::Binary,
                            desired_reply_type: MessageType::Response,
                        },
                    )?)
                    .expects_response(1000) // TODO figure this out
                    .blob_bytes(bincode::serialize(&ProveRequest {
                        elf: ELF.to_vec(),
                        input: stdin,
                    })?)
                    .send()
                    .unwrap();
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
    if http_request.bound_path(Some(&our.process.to_string())) != "/rpc" {
        http::send_response(
            http::StatusCode::NOT_FOUND,
            None,
            "Not Found".to_string().as_bytes().to_vec(),
        );
        return Ok(());
    }
    match http_request.method()?.as_str() {
        // on GET: view balances
        "GET" => {
            println!("sequencer: got GET request, returning balances...");
            http::send_response(
                http::StatusCode::OK,
                Some(HashMap::from([(
                    String::from("Content-Type"),
                    String::from("application/json"),
                )])),
                serde_json::to_vec(&state.balances)?,
            );
            Ok(())
        }
        // on POST: new transaction
        // TODO think we may need more RPC methods here
        "POST" => {
            println!("sequencer: got POST request, handling transaction...");
            let Some(blob) = get_blob() else {
                return Ok(http::send_response(
                    http::StatusCode::BAD_REQUEST,
                    None,
                    vec![],
                ));
            };
            let tx = serde_json::from_slice::<WrappedTransaction>(&blob.bytes)?;

            chain_event_loop(tx, state)?;
            save_rollup_state(state);
            // TODO propagate tx to DA layer
            http::send_response(
                http::StatusCode::OK,
                None, // TODO application/json
                "todo send tx receipt here".to_string().as_bytes().to_vec(),
            );

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
