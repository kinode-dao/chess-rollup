use serde::{Deserialize, Serialize};
use std::str::FromStr;
use sp1_core::{SP1Prover, SP1Stdin};

use kinode_process_lib::{
    await_message, call_init, println, lazy_load_blob, Address, Message, ProcessId, Request, Response,
};

wit_bindgen::generate!({
    path: "wit",
    world: "process",
    exports: {
        world: Component,
    },
});

fn init(our: Address) {
    println!("{}: begin", our.package());

    loop {
        match handle_message(&our, &mut message_archive) {
            Ok(()) => {}
            Err(e) => {
                println!("prover: error: {:?}", e);
            }
        };
    }
}

call_init!(init);

fn handle_message(our: &Address, message_archive: &mut MessageArchive) -> anyhow::Result<()> {
    let message = await_message()?;

    match message {
        Message::Response { .. } => {
            return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
        }
        Message::Request {
            ref source,
            ref body,
            ..
        } => match serde_json::from_slice(body)? {
            // NOTE I'm basically certain that wasm is NOT powerful enough to do this stuff
            // so I'll have to write this as a runtime extension, which is fine for now
            let inputs = bincode::deserialize::<SP1Stdin>(body)?;
            let elf = lazy_load_blob();

            let Ok(proof) = SP1Prover::prove(elf, inputs) else {
                return Err(anyhow::anyhow("sheeeeit"));
            };

            // NOTE could put this in blob but then body would be empty. Not sure if we need metadata about the proof in the body just yet so this works for now
            let _ = Response::new()
                .body(serde_json::to_vec(&proof).unwrap())
                .send();
        },
    }
    Ok(())
}
