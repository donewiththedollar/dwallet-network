use group::PartyID;
use mpc::{Output, Party};
use twopc_mpc::dkg::Protocol;
use crate::dwallet_mpc::dkg::{AsyncProtocol, DKGFirstParty, DKGSecondParty};
use crate::dwallet_mpc::presign::{SignAuxiliaryInput, SignFirstParty};
