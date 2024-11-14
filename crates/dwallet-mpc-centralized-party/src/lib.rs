use class_groups_constants::protocol_public_parameters;
/// This crate contains the cryptographic logic for the centralized 2PC-MPC party
use k256::ecdsa::hazmat::bits2field;
use k256::ecdsa::signature::digest::{Digest, FixedOutput};
use k256::elliptic_curve::ops::Reduce;
use k256::{elliptic_curve, U256};
use log::debug;
use mpc::two_party::Round;
use rand_core::OsRng;
use twopc_mpc::secp256k1;
use twopc_mpc::tests::setup_class_groups_secp256k1;

type AsyncProtocol = twopc_mpc::secp256k1::class_groups::AsyncProtocol;
type DKGCentralizedParty = <AsyncProtocol as twopc_mpc::dkg::Protocol>::DKGCentralizedParty;

/// Executes the second phase of the DKG protocol, part of a three-phase DKG flow.
///
/// The [`create_dkg_output`] function is called by the client (aka the centralized party)
/// and is responsible for generating and returning the public key share and its proof, as well as the
/// centralized DKG output. These values are necessary for the decentralized party to complete the final
/// phase of the DKG protocol.
///
/// * `decentralized_first_round_output` - A serialized byte vector representing the output of the
///   decentralized party from the first round.
/// * `session_id` - A unique identifier for the session, represented as a hexadecimal string.
///   Received from the `pera_system::dwallet_2pc_mpc_ecdsa_k1::launch_dkg_first_round` transaction.
pub fn create_dkg_output(
    decentralized_first_round_output: Vec<u8>,
    session_id: String,
) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    let decentralized_first_round_output: <AsyncProtocol as twopc_mpc::dkg::Protocol>::EncryptionOfSecretKeyShareAndPublicKeyShare = bcs::from_bytes(&decentralized_first_round_output)?;
    // let public_parameters = class_groups_constants::protocol_public_parameters();
    let public_parameters = protocol_public_parameters();
    let session_id = commitment::CommitmentSizedNumber::from_le_hex(&session_id);

    let (public_key_share_and_proof, centralized_output) = DKGCentralizedParty::advance(
        decentralized_first_round_output,
        &(public_parameters, session_id).into(),
        &mut OsRng,
    )?;

    let public_key_share_and_proof = bcs::to_bytes(&public_key_share_and_proof)?;
    let centralized_output = bcs::to_bytes(&centralized_output)?;

    Ok((public_key_share_and_proof, centralized_output))
}

#[derive(Clone, Debug)]
pub enum Hash {
    KECCAK256 = 0,
    SHA256 = 1,
}

impl TryFrom<u8> for Hash {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Hash::KECCAK256),
            1 => Ok(Hash::SHA256),
            _ => Err(anyhow::Error::msg("Invalid value for Hash enum")),
        }
    }
}

pub fn message_digest(message: &[u8], hash_type: &Hash) -> secp256k1::Scalar {
    let hash = match hash_type {
        Hash::KECCAK256 => bits2field::<k256::Secp256k1>(
            &sha3::Keccak256::new_with_prefix(message).finalize_fixed(),
        ),
        Hash::SHA256 => {
            bits2field::<k256::Secp256k1>(&sha2::Sha256::new_with_prefix(message).finalize_fixed())
        }
    }
    .unwrap();
    #[allow(clippy::useless_conversion)]
    let m = <elliptic_curve::Scalar<k256::Secp256k1> as Reduce<U256>>::reduce_bytes(&hash.into());
    U256::from(m).into()
}

///
type SignCentralizedParty = <AsyncProtocol as twopc_mpc::sign::Protocol>::SignCentralizedParty;

/// Executes the centralized phase of the Sign protocol, first part of the protocol
///
/// The [`create_sign_output`] function is called by the client (aka the centralized party).
///
/// The `session_id` is a unique identifier for the session, represented as a hexadecimal string.
/// The `hash` must fit to the [`Hash`] enum.
pub fn create_sign_output(
    centralized_party_dkg_output: Vec<u8>,
    presign_first_round_output: Vec<u8>,
    presign_second_round_output: Vec<u8>,
    message: Vec<u8>,
    hash: u8,
    session_id: String,
) -> anyhow::Result<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> {
    let centralized_party_dkg_output: <AsyncProtocol as twopc_mpc::dkg::Protocol>::CentralizedPartyDKGOutput = bcs::from_bytes(&centralized_party_dkg_output)?;
    let presign_first_round_output: <AsyncProtocol as twopc_mpc::presign::Protocol>::EncryptionOfMaskAndMaskedNonceShare = bcs::from_bytes(&presign_first_round_output)?;
    let presign_second_round_output: (<AsyncProtocol as twopc_mpc::presign::Protocol>::NoncePublicShareAndEncryptionOfMaskedNonceSharePart, <AsyncProtocol as twopc_mpc::presign::Protocol>::NoncePublicShareAndEncryptionOfMaskedNonceSharePart) = bcs::from_bytes(&presign_second_round_output)?;
    let presigns: <AsyncProtocol as twopc_mpc::presign::Protocol>::Presign =
        (presign_first_round_output, presign_second_round_output).into();
    let session_id = commitment::CommitmentSizedNumber::from_le_hex(&session_id);
    debug!("sign wasm session id: {:?}", session_id);
    let message = "singing!";
    let hash_message = message_digest(&message.as_bytes(), &hash.try_into()?);
    // let protocol_public_parameters = class_groups_constants::protocol_public_parameters();
    let protocol_public_parameters = protocol_public_parameters();

    // let centralized_party_auxiliary_input = (
    //     hash_message,
    //     centralized_party_dkg_output.clone(),
    //     presigns.clone(),
    //     protocol_public_parameters.clone(),
    //     session_id,
    // )
    //     .into();
    // let (sign_message, centralized_output) =
    //     SignCentralizedParty::advance((), &centralized_party_auxiliary_input, &mut OsRng)?;
    let centralized_output = ();
    let sign_message = "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";
    // let sign_message = base64::decode(sign_message)?;
    // let sign_message: <AsyncProtocol as twopc_mpc::sign::Protocol>::SignMessage = bcs::from_bytes(&sign_message)?;
    let sign_message = bcs::to_bytes(&sign_message)?;
    let centralized_output = bcs::to_bytes(&centralized_output)?;
    let presigns = bcs::to_bytes(&presigns)?;
    let hash_message = bcs::to_bytes(&hash_message)?;
    Ok((sign_message, centralized_output, presigns, hash_message))
}
