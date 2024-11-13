use dwallet_mpc::{Hash, message_digest};

fn main() {
    let a = vec![1, 2, 3, 4, 5];
    let digest = message_digest(&a, &Hash::SHA256);
    println!("{:?}", base64::encode(bcs::to_bytes(&digest).unwrap()));
}