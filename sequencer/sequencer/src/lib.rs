#![feature(let_chains)]
use kinode_process_lib::eth;
use kinode_process_lib::kernel_types::MessageType;
use kinode_process_lib::{
    await_message, call_init, get_blob, get_typed_state, http, println, set_state,
    vfs::{create_drive, create_file},
    Address, Message, Request,
};
use serde::{Deserialize, Serialize};
use sp1_core::SP1Stdin;
use std::collections::HashMap;

mod bridge_lib;
use bridge_lib::{get_old_logs, handle_log, subscribe_to_logs};
mod chess_engine;
use chess_engine::{ChessRollupState, ChessState, ChessTransactions};
mod prover_types;
use prover_types::ProveRequest;
mod rollup_lib;
use rollup_lib::*;

const ELF: &[u8] = include_bytes!("../../../elf_program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AdminActions {
    Prove,
    // Disperse,
}

pub fn save_rollup_state(state: &ChessRollupState) {
    set_state(&bincode::serialize(&state).unwrap());
    // NOTE this function also needs to include logic for pushing to some DA layer
}

pub fn load_rollup_state() -> ChessRollupState {
    match get_typed_state(|bytes| Ok(bincode::deserialize::<ChessRollupState>(bytes)?)) {
        Some(rs) => rs,
        None => ChessRollupState::default(),
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

    let mut state: ChessRollupState = load_rollup_state();
    let mut connection: Option<u32> = None;

    let eth_provider = eth::Provider::new(11155111, 5); // sepolia, 5s timeout

    get_old_logs(&eth_provider, &mut state);
    subscribe_to_logs(&eth_provider);

    main_loop(&our, &mut state, &mut connection);
}

fn main_loop(our: &Address, state: &mut ChessRollupState, connection: &mut Option<u32>) {
    loop {
        match await_message() {
            Err(send_error) => {
                println!("{our}: got network error: {send_error:?}");
                continue;
            }
            Ok(message) => match handle_message(&our, &message, state, connection) {
                Ok(()) => continue,
                Err(e) => println!("{our}: error handling request: {:?}", e),
            },
        }
    }
}

fn handle_message(
    our: &Address,
    message: &Message,
    state: &mut ChessRollupState,
    connection: &mut Option<u32>,
) -> anyhow::Result<()> {
    // no responses
    if !message.is_request() {
        return Ok(());
    }
    if message.source().node != our.node {
        println!("got cross rollup message, implementation is TODO");
        // first verify that this message was posted to DA
        // then sequence it
        return Ok(());
    }
    return match message.source().process.to_string().as_str() {
        "http_server:distro:sys" => {
            return handle_http_request(our, state, connection, message);
        }
        "eth:distro:sys" => {
            println!("got eth message");
            let Ok(Ok(eth::EthSub { result, .. })) =
                serde_json::from_slice::<eth::EthSubResult>(message.body())
            else {
                return Err(anyhow::anyhow!("sequencer: got invalid message"));
            };
            let eth::SubscriptionResult::Log(log) = result else {
                panic!("expected log");
            };
            handle_log(state, &log)
        }
        _ => handle_admin_message(message, state, connection),
    };
}

/// Handle HTTP requests from our own frontend.
fn handle_http_request(
    our: &Address,
    state: &mut ChessRollupState,
    connection: &mut Option<u32>,
    message: &Message,
) -> anyhow::Result<()> {
    match serde_json::from_slice::<http::HttpServerRequest>(message.body())? {
        http::HttpServerRequest::Http(ref incoming) => {
            // TODO fix this
            // if http_request.bound_path(Some(&our.process.to_string())) != "/rpc" {
            //     http::send_response(
            //         http::StatusCode::NOT_FOUND,
            //         None,
            //         "Not Found".to_string().as_bytes().to_vec(),
            //     );
            //     return Ok(());
            // }
            match incoming.method()?.as_str() {
                // on GET: view state
                // TODO: better read api
                "GET" => {
                    println!("sequencer: got GET request, returning balances...");
                    http::send_response(
                        http::StatusCode::OK,
                        Some(HashMap::from([(
                            String::from("Content-Type"),
                            String::from("application/json"),
                        )])),
                        serde_json::to_vec(&state)?,
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
                    let tx = serde_json::from_slice::<WrappedTransaction<ChessTransactions>>(
                        &blob.bytes,
                    )?;

                    state.execute(tx)?;
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
        http::HttpServerRequest::WebSocketOpen { ref channel_id, .. } => {
            println!("sequencer: connected to prover_extension");
            *connection = Some(*channel_id);
            Ok(())
        }
        http::HttpServerRequest::WebSocketClose(ref channel_id) => {
            if connection.unwrap_or(0) != *channel_id {
                return Err(anyhow::anyhow!("WebSocketClose wrong channel_id"));
            }
            println!("sequencer: dropped connection with prover_extension");
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
                // id,
                // kinode_message_type,
                blob,
                ..
            } = serde_json::from_slice(&blob.bytes)?
            else {
                return Err(anyhow::anyhow!("expected WebSocketExtPushData"));
            };
            let drive_path: String = create_drive(our.package_id(), "proofs", Some(5))?;
            let proof_file = create_file(&format!("{}/proof.json", &drive_path), Some(5))?;
            proof_file.write(&blob)?;
            // TODO verify this proof on some blockchain
            Ok(())
        }
    }
}

fn handle_admin_message(
    message: &Message,
    state: &mut ChessRollupState,
    connection: &mut Option<u32>,
) -> anyhow::Result<()> {
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
                .blob_bytes(bincode::serialize(&ProveRequest {
                    elf: ELF.to_vec(),
                    input: stdin,
                })?)
                .send()
                .unwrap();
            // NOTE the response comes in as a WebSocketPush message
            Ok(())
        } // AdminActions::Disperse => {
          //     // TODO this should probably just happen automatically when Prove is called
          //     let _ = Request::new()
          //         .target(("our", "disperser", "rollup", "goldfinger.os"))
          //         .body(serde_json::to_vec(&DisperserActions::PostBatch(
          //             state.sequenced.clone(),
          //         ))?)
          //         .send()?;
          //     Ok(())
          // }
    }
}
