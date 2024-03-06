use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use sp1_core::SP1Prover;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message::{Binary, Close},
};

use kinode_lib::types::http_server::HttpServerAction;

mod prover_types;
use prover_types::ProveRequest;

type Receiver = mpsc::Receiver<Vec<u8>>;
type Sender = mpsc::Sender<Vec<u8>>;

/// Kinode Python code runner extension
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Kinode port
    #[arg(short, long)]
    port: u16,
}

const LOCALHOST: &str = "ws://localhost";
const PROCESS_ID: &str = "sequencer:rollup:goldfinger.os";
const EVENT_LOOP_CHANNEL_CAPACITY: usize = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let url = format!("{}:{}/{}", LOCALHOST, args.port, PROCESS_ID);
    let (send_to_loop, mut recv_in_loop): (Sender, Receiver) =
        mpsc::channel(EVENT_LOOP_CHANNEL_CAPACITY);
    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    loop {
        tokio::select! {
            Some(message) = read.next() => {
                match message {
                    Ok(Binary(ref request)) => {
                        let request = rmp_serde::from_slice(request)?;
                        prover(request, send_to_loop.clone()).await?;
                    }
                    Ok(Close(_)) => {
                        eprintln!("Server closed the connection");
                        return Err(anyhow::anyhow!("Server closed the connection"));
                    }
                    Err(e) => {
                        eprintln!("Error in receiving message: {}", e);
                        return Err(anyhow::anyhow!("Error in receiving message: {}", e));
                    }
                    _ => {}
                }
            }
            Some(result) = recv_in_loop.recv() => {
                match write.send(Binary(result)).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error in sending message: {}", e);
                    }
                }
            }
        }
    }
}

async fn prover(request: HttpServerAction, send_to_loop: Sender) -> anyhow::Result<()> {
    let HttpServerAction::WebSocketExtPushData {
        id,
        kinode_message_type,
        blob,
    } = request
    else {
        return Err(anyhow::anyhow!("not a WebSocketExtPushData, as expected"));
    };
    let req: ProveRequest = bincode::deserialize(&blob)?;
    let Ok(proof) = SP1Prover::prove(&req.elf, req.input) else {
        return Err(anyhow::anyhow!("error proving request"));
    };

    let result = serde_json::to_vec(&HttpServerAction::WebSocketExtPushData {
        id,
        kinode_message_type,
        blob: serde_json::to_string(&proof).unwrap().into_bytes(),
    })
    .unwrap();
    let _ = send_to_loop.send(result).await;
    Ok(())
}
