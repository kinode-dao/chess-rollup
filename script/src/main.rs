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
    let black_pub = "2bc031cac37ac417b5ba436f8beb89199c52b7996ad855691abe7f6c1248f6d3";

    let white_move_1 = Move {
        san: "6534".to_string(), // e4 in hex
        sig: "db0509435b2645ef7f5526974ba32fb1c8f2a5704ae900128c3a2013a6f98fef98b3f8b8b16d7a35e143f886c85e1b1c37b05673a253cd46d696f9dff91fc702".to_string(),
    };

    let mut stdin = SP1Stdin::new();
    stdin.write(&white_pub);
    stdin.write(&black_pub);
    stdin.write(&vec![white_move_1]);

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
