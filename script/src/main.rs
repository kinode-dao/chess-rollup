use ethers::prelude::*; //core::types::Signature;
                        // use ethers::ethers_signers::Signer;
                        // use ethers::signers::LocalWallet;
use serde::{Deserialize, Serialize};
use sp1_core::{SP1Prover, SP1Stdin, SP1Verifier};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

#[derive(Serialize, Deserialize)]
struct Move {
    san: String,
    sig: String,
}

fn main() {
    let white_priv = "014f081695a25d24a3061cf165bcdeda23e787fa6df4b6a4ed6d59ddeeac7c7f960059693061a45c487f62d9964e6dba39ca65617e6fee10cec4de607507cf71";
    let black_priv = "3805ec1fe0a05d5f0b7597cd20d57f47e59eca9173dc825db92a4f5f935ac0f72bc031cac37ac417b5ba436f8beb89199c52b7996ad855691abe7f6c1248f6d3";

    let white_pub = "960059693061a45c487f62d9964e6dba39ca65617e6fee10cec4de607507cf71";
    let black_pub = "2bc031cac37ac417b5ba436f8beb89199c52b7996ad855691abe7f6c1248f6d3";

    let white_move_1 = Move {
        san: "6534".to_string(), // e4 in hex
        sig: "4b6ee16b3f040d79c7914f5757540b9cc2ae768430215e58bf77fb324ad6eeed6474aa777de0e0e2614935e451b3516c90d1e5c65218acdb842639a5d9c11e046534".to_string(),
    };
    let black_move_1 = Move {
        san: "6335".to_string(), // c5 in hex
        sig: "1f73235d44a6115057d0b03c155b66288fbfe4f65bcdae9e72bce2d2088a65fe9ee3bb2a24e0e2ab07b11ff822b596c09d29bbe60c48b4a4c9c9ac233fe06e0e6335".to_string(),
    };
    let moves = vec![white_move_1, black_move_1];

    let mut stdin = SP1Stdin::new();
    stdin.write(&white_pub);
    stdin.write(&black_pub);
    stdin.write(&moves);

    let mut proof = SP1Prover::prove(ELF, stdin).expect("proving failed");

    // Read output.
    let is_valid_move = proof.stdout.read::<bool>();
    println!("is_valid_move: {}", is_valid_move);

    // Verify proof.
    SP1Verifier::verify(ELF, &proof).expect("verification failed");

    // Save proof.
    proof
        .save("proof-with-io.json")
        .expect("saving proof failed");

    println!("succesfully generated and verified proof for the program!")
}
