use crate::chess_engine::{ChessRollupState, ChessTransactions};
use crate::rollup_lib::{ExecutionEngine, RpcApi, SignedTransaction};
use crate::save_rollup_state;
use kinode_process_lib::{get_blob, http};
use std::collections::HashMap;

/// This is the RPC handler for the chess rollup state.
/// You will notice that it is incredibly simple: the only read operation is an http GET which
/// returns the entire state serialized into json, while the only write operation is an http POST
/// which expects a json-serialized SignedTransaction as input.
impl RpcApi<ChessRollupState, ChessTransactions> for ChessRollupState {
    fn rpc(&mut self, req: &http::IncomingHttpRequest) -> anyhow::Result<()> {
        match req.method()?.as_str() {
            "GET" => {
                // For simplicity, we just return the entire state as the only chain READ operation
                http::send_response(
                    http::StatusCode::OK,
                    Some(HashMap::from([(
                        String::from("Content-Type"),
                        String::from("application/json"),
                    )])),
                    serde_json::to_vec(&self)?,
                );
                Ok(())
            }
            "POST" => {
                // get the blob from the request
                let Some(blob) = get_blob() else {
                    return Ok(http::send_response(
                        http::StatusCode::BAD_REQUEST,
                        None,
                        vec![],
                    ));
                };
                // deserialize the blob into a SignedTransaction
                let tx =
                    serde_json::from_slice::<SignedTransaction<ChessTransactions>>(&blob.bytes)?;

                // execute the transaction, which will propagate any errors like a bad signature or bad move
                self.execute(tx)?;
                save_rollup_state(self);
                // send a confirmation to the frontend that the transaction was sequenced
                http::send_response(
                    http::StatusCode::OK,
                    None,
                    "todo send tx receipt or error here"
                        .to_string()
                        .as_bytes()
                        .to_vec(),
                );

                Ok(())
            }
            // Any other http method will be rejected.
            _ => Ok(http::send_response(
                http::StatusCode::METHOD_NOT_ALLOWED,
                None,
                vec![],
            )),
        }
    }
}
