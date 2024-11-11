use twopc_mpc::tests::setup_class_groups_secp256k1;
use class_groups_constants::{decryption_key, protocol_public_parameters};

fn main() {
    let public = protocol_public_parameters();
    let secret = decryption_key();
    println!("{:?}\n{:?}", public, secret);
}