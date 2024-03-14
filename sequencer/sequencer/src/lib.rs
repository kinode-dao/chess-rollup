#![feature(let_chains)]
use alloy_primitives::{address, FixedBytes, Signature, U256};
use alloy_sol_types::{sol, SolEvent};
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

mod dac;
use dac::*;
mod prover_types;
use prover_types::ProveRequest;
mod tx;
use tx::*;

const ELF: &[u8] = include_bytes!("../../../elf_program/elf/riscv32im-succinct-zkvm-elf");

sol! {
    event DepositMade(uint256 town, address tokenContract, uint256 tokenId,
        address uqbarDest, uint256 amount, uint256 blockNumber, bytes32 prevDepositRoot
    );
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AdminActions {
    Prove,
    Disperse,
}

pub fn save_rollup_state(state: &RollupState) {
    set_state(&bincode::serialize(&state).unwrap());
    // NOTE this function also needs to include logic for pushing to some DA layer
}

pub fn load_rollup_state() -> RollupState {
    match get_typed_state(|bytes| Ok(bincode::deserialize::<RollupState>(bytes)?)) {
        Some(rs) => rs,
        None => RollupState {
            sequenced: Vec::new(),
            balances: HashMap::new(),
            withdrawals: Vec::new(),
            pending_games: HashMap::new(),
            games: HashMap::new(),
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

    let eth_provider = eth::Provider::new(11155111, 5); // sepolia, 5s timeout
    let filter = eth::Filter::new()
        .address(
            "0x8B2FBB3f09123e478b55209Ec533f56D6ee83b8b"
                .parse::<eth::Address>()
                .unwrap(),
        )
        .from_block(5436837)
        .to_block(eth::BlockNumberOrTag::Latest)
        .events(vec![
            "DepositMade(uint256,address,uint256,address,uint256,uint256,bytes32)",
        ]);

    loop {
        match eth_provider.get_logs(&filter) {
            Ok(logs) => {
                for log in logs {
                    match handle_log(&mut state, &log) {
                        Ok(()) => continue,
                        Err(e) => println!("error handling log: {:?}", e),
                    }
                }
                break;
            }
            Err(_) => {
                println!("failed to fetch logs! trying again in 5s...");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        }
    }

    loop {
        match eth_provider.subscribe(1, filter.clone()) {
            Ok(()) => break,
            Err(_) => {
                println!("failed to subscribe to chain! trying again in 5s...");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        }
    }
    println!("subscribed to logs successfully");

    main_loop(&our, &mut state, &mut connection);
}

fn main_loop(our: &Address, state: &mut RollupState, connection: &mut Option<u32>) {
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
    state: &mut RollupState,
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
    } else if message.source().node == our.node && message.source().process == "eth:distro:sys" {
        println!("got eth message");
        let Ok(Ok(eth::EthSub { result, .. })) =
            serde_json::from_slice::<eth::EthSubResult>(message.body())
        else {
            return Err(anyhow::anyhow!("sequencer: got invalid message"));
        };

        let eth::SubscriptionResult::Log(log) = result else {
            panic!("expected log");
        };

        return handle_log(state, &log);
    } else if message.source().node == our.node {
        return handle_admin_message(message, state, connection);
    } else {
        return Err(anyhow::anyhow!("ignoring request"));
    }
}

/// Handle HTTP requests from our own frontend.
fn handle_http_request(
    _our: &Address,
    state: &mut RollupState,
    http_request: &http::IncomingHttpRequest,
) -> anyhow::Result<()> {
    // TODO fix this
    // if http_request.bound_path(Some(&our.process.to_string())) != "/rpc" {
    //     http::send_response(
    //         http::StatusCode::NOT_FOUND,
    //         None,
    //         "Not Found".to_string().as_bytes().to_vec(),
    //     );
    //     return Ok(());
    // }
    match http_request.method()?.as_str() {
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

fn handle_admin_message(
    message: &Message,
    state: &mut RollupState,
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
        }
        AdminActions::Disperse => {
            // TODO this should probably just happen automatically when Prove is called
            let _ = Request::new()
                .target(("our", "disperser", "rollup", "goldfinger.os"))
                .body(serde_json::to_vec(&DisperserActions::PostBatch(
                    state.sequenced.clone(),
                ))?)
                .send()?;
            Ok(())
        }
    }
}

fn handle_log(state: &mut RollupState, log: &eth::Log) -> anyhow::Result<()> {
    // NOTE this ugliness is only because kinode_process_lib::eth is using an old version of alloy. Once it's at 0.6.3/4 we can clear this up
    match FixedBytes::<32>::new(log.topics[0].as_slice().try_into().unwrap()) {
        DepositMade::SIGNATURE_HASH => {
            println!("deposit event");
            // let event = DepositMade::from_log(&log)?;
            let deposit = DepositMade::abi_decode_data(&log.data, true).unwrap();
            let rollup_id = deposit.0;
            let token_contract = deposit.1;
            let token_id = deposit.2;
            let uqbar_dest = deposit.3;
            let amount = deposit.4;
            let _block_number = deposit.5;
            let _prev_deposit_root = deposit.6;
            if rollup_id != U256::ZERO {
                return Err(anyhow::anyhow!("not handling rollup deposits"));
            }
            if token_contract != address!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee") {
                return Err(anyhow::anyhow!("only handling ETH deposits"));
            }
            if token_id != U256::ZERO {
                return Err(anyhow::anyhow!("not handling NFT deposits"));
            }

            state
                .balances
                .entry(uqbar_dest)
                .and_modify(|balance| *balance += amount)
                .or_insert(amount);
            // NOTE that it is impossible to establish a proper sequence of when bridge transactions get inserted
            // relative to other transactions => MEV! (if there is any MEV to be done, which I kind of doubt)
            // the point is that you can prove that these are part of the inputs to the program, and they have to be
            // sequenced at some point before the batch is over
            state.sequenced.push(WrappedTransaction {
                pub_key: uqbar_dest,
                // TODO maybe need to rearchitect bridge transactions because they don't really have a signature
                // you could get a signature from the sequencer? That could work! But at the end of the day it
                // doesn't matter, you don't need to verify it.
                sig: Signature::test_signature(),
                data: TxType::BridgeTokens(amount),
            });
        }
        _ => {
            return Err(anyhow::anyhow!("unknown event"));
        }
    }
    Ok(())
}
