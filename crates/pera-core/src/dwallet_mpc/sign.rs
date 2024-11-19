use group::PartyID;
use mpc::{Party, WeightedThresholdAccessStructure};
use twopc_mpc::dkg::Protocol;

use pera_types::error::PeraResult;

use crate::dwallet_mpc::dkg::{AsyncProtocol, DKGFirstParty, DKGFirstPartyAuxiliaryInputGenerator};
use crate::dwallet_mpc::presign::{FirstSignBytesParty, SignAuxiliaryInput};

impl FirstSignBytesParty {
    pub(crate) fn generate_auxiliary_input(
        session_id: Vec<u8>,
        weighted_threshold_access_structure: WeightedThresholdAccessStructure,
        party_id: PartyID,
        dkg_output: Vec<u8>,
        hashed_message: Vec<u8>,
        presign: Vec<u8>,
        centralized_signed_message: Vec<u8>,
        decryption_key_share_public_parameters: <AsyncProtocol as twopc_mpc::sign::Protocol>::DecryptionKeySharePublicParameters,
    ) -> PeraResult<Vec<u8>> {
        let auxiliary_auxiliary_input = DKGFirstParty::generate_auxiliary_input(
            session_id.clone(),
            party_id,
            weighted_threshold_access_structure,
        );

        let auxiliary = SignAuxiliaryInput::from((
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
            >(&centralized_signed_message)?,
            decryption_key_share_public_parameters,
        ));

        Ok(bcs::to_bytes(&auxiliary)?)
    }
}
