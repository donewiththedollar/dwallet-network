use group::PartyID;
use twopc_mpc::sign::deal_blockchain_secret_shares;
use class_groups_constants::{decryption_key, protocol_public_parameters};

fn main() {
    let (mut decryption_key_shares, decryption_key_share_public_parameters) =
        deal_blockchain_secret_shares(
            3, 4,
            protocol_public_parameters(), decryption_key()
        );
    let clone = decryption_key_share_public_parameters.clone();
    println!("Decryption key shares: {:?}", clone);
}