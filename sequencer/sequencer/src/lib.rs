#![feature(let_chains)]
use kinode_process_lib::{
    await_message, call_init, get_blob, get_typed_state, http, println, set_state, Address, Message,
};
use std::collections::HashMap;

mod tx;
use tx::*;

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
