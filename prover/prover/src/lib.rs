use sp1_core::{SP1Prover, SP1Stdin};

use kinode_process_lib::{await_message, call_init, get_blob, println, Address, Message, Response};

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
        match handle_message(&our) {
            Ok(()) => {}
            Err(e) => {
                println!("prover: error: {:?}", e);
            }
        };
    }
}

call_init!(init);

fn handle_message(_our: &Address) -> anyhow::Result<()> {
    let message = await_message()?;

    let Message::Request { ref body, .. } = message else {
        return Err(anyhow::anyhow!("unexpected Response: {:?}", message));
    };
    // TODO filter based on source, need to pay a token, etc.

    // NOTE I'm basically certain that wasm is NOT powerful enough to do this stuff
    // so I'll have to write this as a runtime extension, which is fine for now
    let inputs = bincode::deserialize::<SP1Stdin>(body)?;
    let elf = get_blob().ok_or(anyhow::anyhow!("no blob"))?;

    let Ok(proof) = SP1Prover::prove(&elf.bytes, inputs) else {
        return Err(anyhow::anyhow!("couldn't prove"));
    };

    // NOTE could put this in blob but then body would be empty. Not sure if we need metadata about the proof in the body just yet so this works for now
    let _ = Response::new()
        .body(serde_json::to_vec(&proof).unwrap())
        .send();
    Ok(())
}
