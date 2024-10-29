use log::{debug, log};
use mpc::two_party;
use mpc::two_party::Round;
use rand_core::OsRng;
use serde_json::from_slice;


type AsyncProtocol = twopc_mpc::secp256k1::class_groups::AsyncProtocol;
type DKGCentralizedParty = <AsyncProtocol as twopc_mpc::dkg::Protocol>::DKGCentralizedParty;

pub fn create_centralized_output(first_round_output: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let base64 = "AOPuPgkDuw4/b7Rg1JdbT3J4Nx9vdx7lIXTCdBQ/MGqcUtYAvyjJgWODdhpTgRBlsVD8rwHORsLBBNYCWTpCprP0ZgCBQZwGHTdEqdX3E6JgVyeKD1qtR/Y6whgsz7pL0cMtzDYOq26/Lj86KBnBCCQFLBwVAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABi/RyLEQUWyR83piolMCsMqlutGw0Eyc+XSEIaIknexWSzPHbM/3GK9WOnU7I+A707psTDvStsi+gZs7hJ/1s+qSREpd9KSqanawFj1kni98tcfg25tHDomlLqfbmujkvy8BySKdX+Daluy5svOAxABSoCA8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAN6GirVzdg9otxrT/JXV7oDs5eSIT2F4u6ScmHTlHQi4QEnsPkzRtVl75iAjbYXqbwdNtcuvClmIanHf+J1ciZditKGZQdx426Hj1kLVM2ChE5z47/9xS2oovl5eqJ3tk+A+f5hKU9JLzX1SgGN5fU49buxQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAPf57UdiFd4LfiohMYiW8tUMo2QlX7crdi28Q/lS8YSA/vjR0dzDDy6d1cQNY+ni4i/p+n1WSO+5bDaDYT1ntKOLhoB9Jh/crquMkx/FfX7vWySwhfrOTVjiI0hLPzU/NcGgUQ1ETEfoOkefbYGeBHyU/9S2AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAhA6addmKxv+yCMcFuUtjf4/JwlHmgpUOLBCSdsdaotmJT";
    let output = base64::decode(base64).unwrap();
    debug!("{:?}", output == first_round_output);
    let a = output == first_round_output;
    debug!("1");
    let first_round_output: <AsyncProtocol as twopc_mpc::dkg::Protocol>::EncryptionOfSecretKeyShareAndPublicKeyShare = from_slice(&first_round_output)?;
    debug!("2");
    let pp = class_groups_constants::protocol_public_parameters()?;
    debug!("1");
    let session_id = commitment::CommitmentSizedNumber::from_u8(8);
    debug!("1");
    let auxiliary_input = (pp, session_id).into();
    debug!("1");
    let (outgoing_message, a) =
        DKGCentralizedParty::advance(first_round_output, &auxiliary_input, &mut OsRng)?;
    debug!("1");
    let outgoing_message = bcs::to_bytes(&outgoing_message)?;
    debug!("1");
    let a = hex::encode(&outgoing_message);
    debug!("1");
    Ok(outgoing_message)
}

pub fn no_code_in_wasm(input: Vec<u8>) -> Vec<u8> {
    let output = match create_centralized_output(input) {
        Ok(output) => output,
        Err(e) => {
            debug!("{:?}", e);
            return vec![1, 2];
        }
    };
    output
}
