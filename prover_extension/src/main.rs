use hex_literal::hex;
use serde::{Deserialize, Serialize};
use sp1_core::{SP1Prover, SP1Stdin, SP1Verifier};
use std::collections::HashMap;

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Serialize, Deserialize)]
struct WrappedTransaction {
    // all these are hex strings, maybe move to alloy types at some point
    pub_key: String,
    sig: String,
    data: String, // hex string
                  // TODO probably need to add nonces, value, gas, gasPrice, gasLimit, ... but whatever
                  // I think we could use eth_sendRawTransaction to just send arbitrary bytes to a sequencer
}

#[derive(Serialize, Deserialize)]
struct BridgeTokens {
    amount: u64,
}

#[derive(Serialize, Deserialize)]
struct InitializeGame {
    white: String,
    black: String,
    wager: u64,
}

#[derive(Serialize, Deserialize)]
struct Move {
    game: (String, String),
    san: String,
}

#[derive(Serialize, Deserialize)]
enum TxType {
    BridgeTokens(BridgeTokens),
    InitializeGame(InitializeGame),
    Move(Move),
}

fn main() {
    // let white_pub = "562099bd28942798feff232b5601f2de46ab234ed121d6c9edd11375dd1c36a0";
    // let black_pub = "9446cce349ab15d3740bfd4e10c6702d1bacf602149a730e6c8c10228ae683df";

    // let white_move_1 = Move {
    //     san: "6534".to_string(), // e4 in hex
    //     sig: "db0509435b2645ef7f5526974ba32fb1c8f2a5704ae900128c3a2013a6f98fef98b3f8b8b16d7a35e143f886c85e1b1c37b05673a253cd46d696f9dff91fc702".to_string(),
    // };

    // let black_move_1 = Move {
    //     san: "6335".to_string(), // c5 in hex
    //     sig: "801341a63d1a1c42893a7b90286632b6a826ec967241d4bfb2b3fcd86728dc7bd3272599ccfd496c872cf704215b01c72a0398cb2a21c1596beb0f05c13f640e".to_string(),
    // };

    // let mut stdin = SP1Stdin::new();
    // stdin.write(&white_pub);
    // stdin.write(&black_pub);
    // stdin.write(&vec![white_move_1, black_move_1]);
    let balances: HashMap<String, u64> = HashMap::new();
    let games: HashMap<(String, String), (String, u64)> = HashMap::new(); // (white, black) -> (fen, wager)
    let tx_1 = WrappedTransaction {
        pub_key: "8613146c8cbde0eb9a3b15766b873580e61971d525399e03386c3aa3fd38cfd3".to_string(),
        sig: "f5088a9b1efee58d88260aa1b2c2fb3a788d3534681c2d9a799909008fd1ae25f5f758d228ca7253b6722bdb2b53ceffdf89d2aab9eed0fefa2edc42756c1206".to_string(),
        data: "000000006400000000000000".to_string(),
    };

    let tx_2 = WrappedTransaction {
        pub_key: "27bb473c8ffeaa0fd9ca4f4bb05b7c7c121b52e60a0b63933e68a40caee6849a".to_string(),
        sig: "d0ab5dae6abfec1c1297e7794d8a6764207b7d5383694fe2e698072173e92f0ad6b64dbd3c642125648aa554a7eb32975c180ca1efe80d7a0f9a780745440201".to_string(),
        data: "000000006400000000000000".to_string(),
    };
    let mut stdin = SP1Stdin::new();
    stdin.write(&balances);
    stdin.write(&games);
    stdin.write(&vec![tx_1, tx_2]);

    let mut proof = SP1Prover::prove(ELF, stdin).expect("proving failed");

    // Read output.
    let out1 = proof.stdout.read::<String>();
    println!("any errors: {}", out1);
    let out2 = proof.stdout.read::<String>();
    println!("any errors: {}", out2);

    // Verify proof.
    SP1Verifier::verify(ELF, &proof).expect("verification failed");

    // Save proof.
    proof
        .save("proof-with-io.json")
        .expect("saving proof failed");

    println!("succesfully generated and verified proof for the program!")
}
