use log::debug;
use dwallet_mpc::create_sign_output;

fn main() {
    let session_id  = "b34f83539d7453106808a155f0d438405ce6eaa4db8a9bd614d8bda1583ed324";
    let session_id = commitment::CommitmentSizedNumber::from_le_hex(&session_id);
    println!("sign wasm session id: {:?}", session_id);
}