use group::PartyID;
use mpc::{Output, Party};
use twopc_mpc::dkg::Protocol;
use crate::dwallet_mpc::dkg::{AsyncProtocol, DKGFirstParty, DKGSecondParty};
use crate::dwallet_mpc::presign::SignFirstParty;

trait SignPartyAuxiliaryInputGenerator: Party {
    fn generate_auxiliary_input(
        number_of_parties: u16,
        party_id: PartyID,
        first_round_output: <DKGFirstParty as Party>::Output,
        centralized_party_public_key_share: <AsyncProtocol as Protocol>::PublicKeyShareAndProof,
        session_is: Vec<u8>,
    ) -> Self::AuxiliaryInput;
}

impl SignPartyAuxiliaryInputGenerator for SignFirstParty {
    fn generate_auxiliary_input(
        number_of_parties: u16,
        party_id: PartyID,
        first_round_output: <DKGFirstParty as Party>::Output,
        centralized_party_public_key_share: <AsyncProtocol as Protocol>::PublicKeyShareAndProof,
        session_id: Vec<u8>,
    ) -> Self::AuxiliaryInput {
        todo!("Implement this function")
    }
}
