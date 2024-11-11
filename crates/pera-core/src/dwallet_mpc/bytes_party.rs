//! The `bytes_party` module defines the API for managing MPC parties within the MPC manager.
//! This module wraps the various MPC parties, providing an interface
//! to progress each party through the rounds of the MPC protocol as needed.
//!
//! The `BytesParty` trait enables the MPC manager to seamlessly advance the `MPCParty`
//! instances to the next round.

use crate::dwallet_mpc::dkg::{AsyncProtocol, FirstDKGBytesParty, SecondDKGBytesParty};
use crate::dwallet_mpc::mpc_events::{
    StartDKGFirstRoundEvent, StartDKGSecondRoundEvent, StartPresignFirstRoundEvent,
    StartPresignSecondRoundEvent, StartSignFirstRoundEvent,
};
use crate::dwallet_mpc::presign::{
    FirstPresignBytesParty, FirstSignBytesParty, PresignFirstParty, PresignSecondParty,
    SecondPresignBytesParty, SignFirstParty,
};
use class_groups_constants::{decryption_key, protocol_public_parameters};
use group::PartyID;
use homomorphic_encryption::GroupsPublicParametersAccessors;
use pera_types::base_types::ObjectID;
use pera_types::error::{PeraError, PeraResult};
use pera_types::event::Event;
use pera_types::messages_dwallet_mpc::{MPCRound, SessionInfo};
use std::collections::HashMap;
use twopc_mpc::sign::create_mock_sign_party;
use twopc_mpc::tests::setup_class_groups_secp256k1;

/// Trait defining the functionality to advance an MPC party to the next round.
///
/// # Arguments
///
/// * `messages` - A hashmap of messages received from other parties, keyed by `PartyID`.
/// * `auxiliary_input` - A serialized vector of auxiliary input data.
///
/// # Returns
///
/// * `Ok(AdvanceResult)` on success, which represents either advancement to the next round
///   or the finalization of the protocol.
/// * `Err(twopc_mpc::Error)` if an error occurs.
pub trait BytesParty: Sync + Send {
    fn advance(
        self,
        messages: HashMap<PartyID, Vec<u8>>,
        auxiliary_input: Vec<u8>,
    ) -> PeraResult<AdvanceResult>;
}

/// Represents the outcome of advancing an MPC party to the next round.
///
/// This enum indicates whether the party should advance to the next round or
/// finalize its protocol execution.
pub enum AdvanceResult {
    /// Contains the message to send to other parties and the next `MPCParty` to use.
    Advance((Vec<u8>, MPCParty)),
    /// Indicates that the protocol has completed, containing the final output.
    Finalize(Vec<u8>),
}

/// Enum representing the different parties used in the MPC manager.
pub enum MPCParty {
    /// A placeholder party used as a default. Does not implement the `BytesParty` trait and should never be used.
    DefaultParty,
    /// The party used in the first round of the DKG protocol.
    FirstDKGBytesParty(FirstDKGBytesParty),
    /// The party used in the second round of the DKG protocol.
    SecondDKGBytesParty(SecondDKGBytesParty),
    /// The party used in the first round of the presign protocol.
    FirstPresignBytesParty(FirstPresignBytesParty),
    /// The party used in the second round of the presign protocol.
    SecondPresignBytesParty(SecondPresignBytesParty),

    FirstSignBytesParty(FirstSignBytesParty),
}

/// Default party implementation for `MPCParty`.
///
/// This variant allows the use of `mem::take`, which requires the `Default` trait.
/// The `DefaultParty` variant is used when a specific party has not been set.
impl Default for MPCParty {
    fn default() -> Self {
        MPCParty::DefaultParty
    }
}

impl MPCParty {
    /// Advances the party to the next round by processing incoming messages and auxiliary input.
    /// Returns the next `MPCParty` to use or the final output if the protocol has completed.
    pub fn advance(
        self,
        messages: HashMap<PartyID, Vec<u8>>,
        auxiliary_input: Vec<u8>,
    ) -> PeraResult<AdvanceResult> {
        match self {
            MPCParty::FirstDKGBytesParty(party) => party.advance(messages, auxiliary_input),
            MPCParty::SecondDKGBytesParty(party) => party.advance(messages, auxiliary_input),
            MPCParty::FirstPresignBytesParty(party) => party.advance(messages, auxiliary_input),
            MPCParty::SecondPresignBytesParty(party) => party.advance(messages, auxiliary_input),
            MPCParty::DefaultParty => Err(PeraError::InternalDWalletMPCError),
            MPCParty::FirstSignBytesParty(party) => party.advance(messages, auxiliary_input),
        }
    }

    /// Parses an `Event` to extract the corresponding `MPCParty`, auxiliary input, and session information.
    /// When `Ok(None)` is returned the event type does not correspond to any known MPC rounds.
    pub fn from_event(
        event: &Event,
        number_of_parties: u16,
        party_id: PartyID,
    ) -> PeraResult<Option<(Self, Vec<u8>, SessionInfo)>> {
        if event.type_ == StartDKGFirstRoundEvent::type_() {
            let deserialized_event: StartDKGFirstRoundEvent = bcs::from_bytes(&event.contents)?;
            return Ok(Some((
                MPCParty::FirstDKGBytesParty(FirstDKGBytesParty {
                    party: <AsyncProtocol as twopc_mpc::dkg::Protocol>::EncryptionOfSecretKeyShareRoundParty::default()
                }),
                FirstDKGBytesParty::generate_auxiliary_input(number_of_parties, party_id, deserialized_event.session_id.bytes.to_vec()),
                SessionInfo {
                    session_id: deserialized_event.session_id.bytes,
                    initiating_user_address: deserialized_event.sender,
                    dwallet_cap_id: deserialized_event.dwallet_cap_id.bytes,
                    mpc_round: MPCRound::DKGFirst,
                },
            )));
        } else if event.type_ == StartDKGSecondRoundEvent::type_() {
            let deserialized_event: StartDKGSecondRoundEvent = bcs::from_bytes(&event.contents)?;
            return Ok(Some((
                MPCParty::SecondDKGBytesParty(SecondDKGBytesParty {
                    party: <AsyncProtocol as twopc_mpc::dkg::Protocol>::ProofVerificationRoundParty::default()
                }),
                SecondDKGBytesParty::generate_auxiliary_input(
                    number_of_parties, party_id,
                    deserialized_event.first_round_output,
                    deserialized_event.public_key_share_and_proof,
                    deserialized_event.first_round_session_id.bytes.to_vec(),
                ).map_err(|_| PeraError::DWalletMPCInvalidUserInput)?,
                SessionInfo {
                    session_id: ObjectID::from(deserialized_event.session_id),
                    initiating_user_address: deserialized_event.sender,
                    dwallet_cap_id: deserialized_event.dwallet_cap_id.bytes,
                    mpc_round: MPCRound::DKGSecond,
                },
            )));
        } else if event.type_ == StartPresignFirstRoundEvent::type_() {
            let deserialized_event: StartPresignFirstRoundEvent = bcs::from_bytes(&event.contents)?;
            return Ok(Some((
                MPCParty::FirstPresignBytesParty(FirstPresignBytesParty {
                    party: PresignFirstParty::default(),
                }),
                FirstPresignBytesParty::generate_auxiliary_input(
                    deserialized_event.session_id.bytes.to_vec(),
                    number_of_parties,
                    party_id,
                    deserialized_event.dkg_output.clone(),
                ),
                SessionInfo {
                    session_id: deserialized_event.session_id.bytes,
                    initiating_user_address: deserialized_event.sender,
                    dwallet_cap_id: deserialized_event.dwallet_cap_id.bytes,
                    mpc_round: MPCRound::PresignFirst(
                        deserialized_event.dwallet_id.bytes,
                        deserialized_event.dkg_output,
                    ),
                },
            )));
        } else if event.type_ == StartPresignSecondRoundEvent::type_() {
            let deserialized_event: StartPresignSecondRoundEvent =
                bcs::from_bytes(&event.contents)?;
            return Ok(Some((
                MPCParty::SecondPresignBytesParty(SecondPresignBytesParty {
                    party: PresignSecondParty::default(),
                }),
                SecondPresignBytesParty::generate_auxiliary_input(
                    deserialized_event.first_round_session_id.bytes.to_vec(),
                    number_of_parties,
                    party_id,
                    deserialized_event.dkg_output,
                    deserialized_event.first_round_output.clone(),
                ),
                SessionInfo {
                    session_id: deserialized_event.session_id.bytes,
                    initiating_user_address: deserialized_event.sender,
                    dwallet_cap_id: deserialized_event.dwallet_cap_id.bytes,
                    mpc_round: MPCRound::PresignSecond(
                        deserialized_event.dwallet_id.bytes,
                        deserialized_event.first_round_output,
                    ),
                },
            )));
        } else if event.type_ == StartSignFirstRoundEvent::type_() {
            let deserialized_event: StartSignFirstRoundEvent = bcs::from_bytes(&event.contents)?;
            let threshold_number_of_parties = ((number_of_parties * 2) + 2) / 3;
            let (party, public_parameters) = create_mock_sign_party(
                party_id,
                threshold_number_of_parties,
                number_of_parties,
                protocol_public_parameters(),
                decryption_key(),
            );
            return Ok(Some((
                MPCParty::FirstSignBytesParty(FirstSignBytesParty { party }),
                FirstSignBytesParty::generate_auxiliary_input(
                    deserialized_event.presign_session_id.bytes.to_vec(),
                    number_of_parties,
                    party_id,
                    deserialized_event.dkg_output,
                    deserialized_event.hashed_message.clone(),
                    deserialized_event.presign.clone(),
                    deserialized_event.centralized_signed_message.clone(),
                    public_parameters,
                )?,
                SessionInfo {
                    session_id: deserialized_event.presign_session_id.bytes,
                    initiating_user_address: deserialized_event.sender,
                    dwallet_cap_id: deserialized_event.dwallet_cap_id.bytes,
                    mpc_round: MPCRound::Sign,
                },
            )));
        }
        Ok(None)
    }
}
