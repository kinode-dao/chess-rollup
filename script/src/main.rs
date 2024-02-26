use serde::{Deserialize, Serialize};
use sp1_core::{SP1Prover, SP1Stdin, SP1Verifier};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Serialize, Deserialize)]
struct Move {
    san: String,
    sig: String,
}

fn main() {
    let white_pub = "562099bd28942798feff232b5601f2de46ab234ed121d6c9edd11375dd1c36a0";
    let black_pub = "9446cce349ab15d3740bfd4e10c6702d1bacf602149a730e6c8c10228ae683df";

    let white_move_1 = Move {
        san: "6534".to_string(), // e4 in hex
        sig: "db0509435b2645ef7f5526974ba32fb1c8f2a5704ae900128c3a2013a6f98fef98b3f8b8b16d7a35e143f886c85e1b1c37b05673a253cd46d696f9dff91fc702".to_string(),
    };

    let black_move_1 = Move {
        san: "6335".to_string(), // c5 in hex
        sig: "801341a63d1a1c42893a7b90286632b6a826ec967241d4bfb2b3fcd86728dc7bd3272599ccfd496c872cf704215b01c72a0398cb2a21c1596beb0f05c13f640e".to_string(),
    };

    let mut stdin = SP1Stdin::new();
    stdin.write(&white_pub);
    stdin.write(&black_pub);
    stdin.write(&vec![white_move_1, black_move_1]);

    let mut proof = SP1Prover::prove(ELF, stdin).expect("proving failed");

    // Read output.
    let is_valid_move = proof.stdout.read::<String>();
    println!("any errors: {}", is_valid_move);

    // Verify proof.
    SP1Verifier::verify(ELF, &proof).expect("verification failed");

    // Save proof.
    proof
        .save("proof-with-io.json")
        .expect("saving proof failed");

    println!("succesfully generated and verified proof for the program!")
}
