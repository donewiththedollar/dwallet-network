use crate::dwallet_mpc::dkg::{AsyncProtocol, DKGFirstParty, DKGSecondParty};
use crate::dwallet_mpc::presign::{
    FirstSignBytesParty, PresignSecondParty, SignAuxiliaryInput, SignFirstParty,
};
use group::PartyID;
use mpc::{Output, Party};
use twopc_mpc::class_groups::Presign;
use twopc_mpc::dkg::Protocol;
use twopc_mpc::paillier::Protocol;

impl FirstSignBytesParty {
    pub(crate) fn generate_auxiliary_input(
        session_id: Vec<u8>,
        number_of_parties: u16,
        party_id: PartyID,
        dkg_output: Vec<u8>,
        hashed_message: Vec<u8>,
        first_round_output: Vec<u8>,
    ) -> Vec<u8> {
        let first_round_output = bcs::from_bytes(&first_round_output).unwrap();
        let auxiliary_auxiliary_input =
            crate::dwallet_mpc::dkg::DKGFirstParty::generate_auxiliary_input(
                session_id.clone(),
                number_of_parties,
                party_id,
            );

        (
            auxiliary_auxiliary_input,
            bcs::from_bytes::<<AsyncProtocol as twopc_mpc::sign::Protocol>::Message>(),
            bcs::from_bytes::<
                <AsyncProtocol as twopc_mpc::sign::Protocol>::DecentralizedPartyDKGOutput,
            >,
            bcs::from_bytes::<
                <AsyncProtocol as twopc_mpc::sign::Protocol>::Presign,
            >,
            bcs::from_bytes::<
                <AsyncProtocol as twopc_mpc::sign::Protocol>::SignMessage,
            >,
            bcs::from_bytes::<
                <AsyncProtocol as twopc_mpc::sign::Protocol>::DecryptionKeySharePublicParameters,
            >,
        ).into()
    }
}

// <(Self::EncryptionOfSecretKeyShareRoundAuxiliaryInput, Self::Message, Self::DecentralizedPartyDKGOutput, Self::Presign, Self::SignMessage, Self::DecryptionKeySharePublicParameters)>