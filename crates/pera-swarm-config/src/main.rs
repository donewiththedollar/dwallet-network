use twopc_mpc::setup_class_groups_secp256k1;

fn main() {
    let (protocol_public_parameters, decryption_key) = setup_class_groups_secp256k1();
    println!(
        "Decryption key: {:?}",
        &base64::encode(bcs::to_bytes(&decryption_key.decryption_key).unwrap())
    );
    println!(
        "Encryption key: {:?}",
        &base64::encode(bcs::to_bytes(&decryption_key.encryption_key).unwrap())
    );
    println!(
        "Public parameters: {:?}",
        &base64::encode(bcs::to_bytes(&protocol_public_parameters).unwrap())
    );
}
