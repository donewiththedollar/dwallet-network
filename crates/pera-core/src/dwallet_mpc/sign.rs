use group::PartyID;
use mpc::{Advance, Party};
use twopc_mpc::dkg::Protocol;

use pera_types::error::{PeraError, PeraResult};
use std::collections::HashMap;
use crate::dwallet_mpc::bytes_party::{AdvanceResult, BytesParty, MPCParty};
use crate::dwallet_mpc::dkg::{AsyncProtocol, DKGFirstParty, DKGFirstPartyAuxiliaryInputGenerator};
use crate::dwallet_mpc::mpc_manager::twopc_error_to_pera_error;
use crate::dwallet_mpc::presign;

impl FirstSignBytesParty {
    pub(crate) fn generate_auxiliary_input(
        session_id: Vec<u8>,
        number_of_parties: u16,
        party_id: PartyID,
        dkg_output: Vec<u8>,
        hashed_message: Vec<u8>,
        presign: Vec<u8>,
        centralized_signed_message: Vec<u8>,
        decryption_key_share_public_parameters: <AsyncProtocol as twopc_mpc::sign::Protocol>::DecryptionKeySharePublicParameters,
    ) -> PeraResult<Vec<u8>> {
        let auxiliary_auxiliary_input = DKGFirstParty::generate_auxiliary_input(
            session_id.clone(),
            number_of_parties,
            party_id,
        );

        let sign_message: &str = bcs::from_bytes(&centralized_signed_message)?;
        let sign_message = base64::decode(&sign_message).unwrap();

        let auxiliary: <AsyncProtocol as twopc_mpc::sign::Protocol>::SignDecentralizedPartyAuxiliaryInput = <AsyncProtocol as twopc_mpc::sign::Protocol>::SignDecentralizedPartyAuxiliaryInput::from((
            auxiliary_auxiliary_input,
            bcs::from_bytes::<<AsyncProtocol as twopc_mpc::sign::Protocol>::Message>(&hashed_message)?,
            bcs::from_bytes::<
                <AsyncProtocol as Protocol>::DecentralizedPartyDKGOutput,
            >(&dkg_output)?,
            bcs::from_bytes::<
                <AsyncProtocol as twopc_mpc::presign::Protocol>::Presign,
            >(&presign)?,
            bcs::from_bytes::<
                <AsyncProtocol as twopc_mpc::sign::Protocol>::SignMessage,
            >(&sign_message)?,
            decryption_key_share_public_parameters,
        ));

        Ok(bcs::to_bytes(&auxiliary)?)
    }
}

pub type SignFirstParty = <presign::AsyncProtocol as twopc_mpc::sign::Protocol>::SignDecentralizedParty;
pub type SignAuxiliaryInput = <presign::AsyncProtocol as twopc_mpc::sign::Protocol>::SignDecentralizedPartyAuxiliaryInput;

/// A wrapper for the second round of the Presign protocol.
///
/// This struct represents the final round of the Presign protocol.
pub struct FirstSignBytesParty {
    pub party: SignFirstParty,
}

impl BytesParty for FirstSignBytesParty {
    fn advance(
        self,
        messages: HashMap<PartyID, Vec<u8>>,
        auxiliary_input_bytes: Vec<u8>,
    ) -> PeraResult<AdvanceResult> {
        let mut auxiliary_input: SignAuxiliaryInput =
            // This is not a validator malicious behaviour, as the authority input is being sent by the initiating user.
            // In this case this MPC session should be cancelled.
            bcs::from_bytes(&auxiliary_input_bytes).map_err(|_| PeraError::DWalletMPCInvalidUserInput)?;

        let messages = messages
            .into_iter()
            .map(|(party_id, message)| {
                let message = bincode::deserialize(&message).unwrap();
                (party_id, message)
            })
            .collect::< HashMap<PartyID, _>>();


        let result = self
            .party
            .advance(
                messages,
                &auxiliary_input,
                &mut rand_core::OsRng,
            );
        if result.is_err() {
            let result = twopc_error_to_pera_error(result.err().unwrap());
            return Err(result);
        }
        match result.map_err(twopc_error_to_pera_error)? {
            mpc::AdvanceResult::Advance((message, new_party)) => {
                Ok(AdvanceResult::Advance((
                    bincode::serialize(&message).unwrap(),
                    // bcs::to_bytes(&message).unwrap(),
                    MPCParty::FirstSignBytesParty(Self { party: new_party }),
                )))},
            mpc::AdvanceResult::Finalize(output) => {
                Ok(AdvanceResult::Finalize(bcs::to_bytes(&output).unwrap()))
            }
        }
    }
}