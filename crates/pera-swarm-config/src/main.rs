use group::PartyID;
use twopc_mpc::setup_class_groups_secp256k1;
use twopc_mpc::sign::deal_blockchain_secret_shares;
use class_groups_constants::{decryption_key, protocol_public_parameters};

fn main() {
    let (protocol_public_parameters, decryption_key) = setup_class_groups_secp256k1();
    println!("Decryption key: {:?}", &base64::encode(bcs::to_bytes(&decryption_key.decryption_key).unwrap()));
    println!("Encryption key: {:?}", &base64::encode(bcs::to_bytes(&decryption_key.encryption_key).unwrap()));
    println!("Public parameters: {:?}", &base64::encode(bcs::to_bytes(&protocol_public_parameters).unwrap()));
}