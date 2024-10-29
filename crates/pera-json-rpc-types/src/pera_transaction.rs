// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: BSD-3-Clause-Clear

use std::fmt::{self, Display, Formatter, Write};
use std::sync::Arc;

use enum_dispatch::enum_dispatch;
use pera_package_resolver::{PackageStore, Resolver};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tabled::{
    builder::Builder as TableBuilder,
    settings::{style::HorizontalLine, Panel as TablePanel, Style as TableStyle},
};

use fastcrypto::encoding::Base64;
use move_binary_format::CompiledModule;
use move_bytecode_utils::module_cache::GetModule;
use move_core_types::annotated_value::MoveTypeLayout;
use move_core_types::identifier::IdentStr;
use move_core_types::language_storage::{ModuleId, StructTag, TypeTag};
use mysten_metrics::monitored_scope;
use pera_json::{primitive_type, PeraJsonValue};
use pera_types::authenticator_state::ActiveJwk;
use pera_types::base_types::{
    EpochId, ObjectID, ObjectRef, PeraAddress, SequenceNumber, TransactionDigest,
};
use pera_types::crypto::PeraSignature;
use pera_types::digests::{
    CheckpointDigest, ConsensusCommitDigest, ObjectDigest, TransactionEventsDigest,
};
use pera_types::effects::{TransactionEffects, TransactionEffectsAPI, TransactionEvents};
use pera_types::error::{ExecutionError, PeraError, PeraResult};
use pera_types::execution_status::ExecutionStatus;
use pera_types::gas::GasCostSummary;
use pera_types::layout_resolver::{get_layout_from_struct_tag, LayoutResolver};
use pera_types::messages_checkpoint::CheckpointSequenceNumber;
use pera_types::messages_consensus::ConsensusDeterminedVersionAssignments;
use pera_types::object::Owner;
use pera_types::parse_pera_type_tag;
use pera_types::pera_serde::Readable;
use pera_types::pera_serde::{
    BigInt, PeraTypeTag as AsPeraTypeTag, SequenceNumber as AsSequenceNumber,
};
use pera_types::quorum_driver_types::ExecuteTransactionRequestType;
use pera_types::signature::GenericSignature;
use pera_types::storage::{DeleteKind, WriteKind};
use pera_types::transaction::{
    Argument, CallArg, ChangeEpoch, Command, EndOfEpochTransactionKind, GenesisObject,
    InputObjectKind, ObjectArg, ProgrammableMoveCall, ProgrammableTransaction, SenderSignedData,
    TransactionData, TransactionDataAPI, TransactionKind,
};
use pera_types::PERA_FRAMEWORK_ADDRESS;

use crate::balance_changes::BalanceChange;
use crate::object_changes::ObjectChange;
use crate::pera_transaction::GenericSignature::Signature;
use crate::{Filter, Page, PeraEvent, PeraObjectRef};

// similar to EpochId of pera-types but BigInt
pub type PeraEpochId = BigInt<u64>;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
#[serde(
    rename_all = "camelCase",
    rename = "TransactionBlockResponseQuery",
    default
)]
pub struct PeraTransactionBlockResponseQuery {
    /// If None, no filter will be applied
    pub filter: Option<TransactionFilter>,
    /// Configure fields to include in the response, by default, only digest is included.
    pub options: Option<PeraTransactionBlockResponseOptions>,
}

impl PeraTransactionBlockResponseQuery {
    pub fn new(
        filter: Option<TransactionFilter>,
        options: Option<PeraTransactionBlockResponseOptions>,
    ) -> Self {
        Self { filter, options }
    }

    pub fn new_with_filter(filter: TransactionFilter) -> Self {
        Self {
            filter: Some(filter),
            options: None,
        }
    }
}

pub type TransactionBlocksPage = Page<PeraTransactionBlockResponse, TransactionDigest>;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Eq, PartialEq, Default)]
#[serde(
    rename_all = "camelCase",
    rename = "TransactionBlockResponseOptions",
    default
)]
pub struct PeraTransactionBlockResponseOptions {
    /// Whether to show transaction input data. Default to be False
    pub show_input: bool,
    /// Whether to show bcs-encoded transaction input data
    pub show_raw_input: bool,
    /// Whether to show transaction effects. Default to be False
    pub show_effects: bool,
    /// Whether to show transaction events. Default to be False
    pub show_events: bool,
    /// Whether to show object_changes. Default to be False
    pub show_object_changes: bool,
    /// Whether to show balance_changes. Default to be False
    pub show_balance_changes: bool,
    /// Whether to show raw transaction effects. Default to be False
    pub show_raw_effects: bool,
}

impl PeraTransactionBlockResponseOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn full_content() -> Self {
        Self {
            show_effects: true,
            show_input: true,
            show_raw_input: true,
            show_events: true,
            show_object_changes: true,
            show_balance_changes: true,
            // This field is added for graphql execution. We keep it false here
            // so current users of `full_content` will not get raw effects unexpectedly.
            show_raw_effects: false,
        }
    }

    pub fn with_input(mut self) -> Self {
        self.show_input = true;
        self
    }

    pub fn with_raw_input(mut self) -> Self {
        self.show_raw_input = true;
        self
    }

    pub fn with_effects(mut self) -> Self {
        self.show_effects = true;
        self
    }

    pub fn with_events(mut self) -> Self {
        self.show_events = true;
        self
    }

    pub fn with_balance_changes(mut self) -> Self {
        self.show_balance_changes = true;
        self
    }

    pub fn with_object_changes(mut self) -> Self {
        self.show_object_changes = true;
        self
    }

    pub fn with_raw_effects(mut self) -> Self {
        self.show_raw_effects = true;
        self
    }

    /// default to return `WaitForEffectsCert` unless some options require
    /// local execution
    pub fn default_execution_request_type(&self) -> ExecuteTransactionRequestType {
        // if people want effects or events, they typically want to wait for local execution
        if self.require_effects() {
            ExecuteTransactionRequestType::WaitForLocalExecution
        } else {
            ExecuteTransactionRequestType::WaitForEffectsCert
        }
    }

    #[deprecated(
        since = "1.33.0",
        note = "Balance and object changes no longer require local execution"
    )]
    pub fn require_local_execution(&self) -> bool {
        self.show_balance_changes || self.show_object_changes
    }

    pub fn require_input(&self) -> bool {
        self.show_input || self.show_raw_input || self.show_object_changes
    }

    pub fn require_effects(&self) -> bool {
        self.show_effects
            || self.show_events
            || self.show_balance_changes
            || self.show_object_changes
            || self.show_raw_effects
    }

    pub fn only_digest(&self) -> bool {
        self == &Self::default()
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, Default)]
#[serde(rename_all = "camelCase", rename = "TransactionBlockResponse")]
pub struct PeraTransactionBlockResponse {
    pub digest: TransactionDigest,
    /// Transaction input data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<PeraTransactionBlock>,
    /// BCS encoded [SenderSignedData] that includes input object references
    /// returns empty array if `show_raw_transaction` is false
    #[serde_as(as = "Base64")]
    #[schemars(with = "Base64")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub raw_transaction: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<PeraTransactionBlockEffects>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<PeraTransactionBlockEvents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_changes: Option<Vec<ObjectChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_changes: Option<Vec<BalanceChange>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<BigInt<u64>>")]
    #[serde_as(as = "Option<BigInt<u64>>")]
    pub timestamp_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_local_execution: Option<bool>,
    /// The checkpoint number when this transaction was included and hence finalized.
    /// This is only returned in the read api, not in the transaction execution api.
    #[schemars(with = "Option<BigInt<u64>>")]
    #[serde_as(as = "Option<BigInt<u64>>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<CheckpointSequenceNumber>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub raw_effects: Vec<u8>,
}

impl PeraTransactionBlockResponse {
    pub fn new(digest: TransactionDigest) -> Self {
        Self {
            digest,
            ..Default::default()
        }
    }

    pub fn status_ok(&self) -> Option<bool> {
        self.effects.as_ref().map(|e| e.status().is_ok())
    }
}

/// We are specifically ignoring events for now until events become more stable.
impl PartialEq for PeraTransactionBlockResponse {
    fn eq(&self, other: &Self) -> bool {
        self.transaction == other.transaction
            && self.effects == other.effects
            && self.timestamp_ms == other.timestamp_ms
            && self.confirmed_local_execution == other.confirmed_local_execution
            && self.checkpoint == other.checkpoint
    }
}

impl Display for PeraTransactionBlockResponse {
    fn fmt(&self, writer: &mut Formatter<'_>) -> fmt::Result {
        writeln!(writer, "Transaction Digest: {}", &self.digest)?;

        if let Some(t) = &self.transaction {
            writeln!(writer, "{}", t)?;
        }

        if let Some(e) = &self.effects {
            writeln!(writer, "{}", e)?;
        }

        if let Some(e) = &self.events {
            writeln!(writer, "{}", e)?;
        }

        if let Some(object_changes) = &self.object_changes {
            let mut builder = TableBuilder::default();
            let (
                mut created,
                mut deleted,
                mut mutated,
                mut published,
                mut transferred,
                mut wrapped,
            ) = (vec![], vec![], vec![], vec![], vec![], vec![]);

            for obj in object_changes {
                match obj {
                    ObjectChange::Created { .. } => created.push(obj),
                    ObjectChange::Deleted { .. } => deleted.push(obj),
                    ObjectChange::Mutated { .. } => mutated.push(obj),
                    ObjectChange::Published { .. } => published.push(obj),
                    ObjectChange::Transferred { .. } => transferred.push(obj),
                    ObjectChange::Wrapped { .. } => wrapped.push(obj),
                };
            }

            write_obj_changes(created, "Created", &mut builder)?;
            write_obj_changes(deleted, "Deleted", &mut builder)?;
            write_obj_changes(mutated, "Mutated", &mut builder)?;
            write_obj_changes(published, "Published", &mut builder)?;
            write_obj_changes(transferred, "Transferred", &mut builder)?;
            write_obj_changes(wrapped, "Wrapped", &mut builder)?;

            let mut table = builder.build();
            table.with(TablePanel::header("Object Changes"));
            table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
                1,
                TableStyle::modern().get_horizontal(),
            )]));
            writeln!(writer, "{}", table)?;
        }

        if let Some(balance_changes) = &self.balance_changes {
            let mut builder = TableBuilder::default();
            for balance in balance_changes {
                builder.push_record(vec![format!("{}", balance)]);
            }
            let mut table = builder.build();
            table.with(TablePanel::header("Balance Changes"));
            table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
                1,
                TableStyle::modern().get_horizontal(),
            )]));
            writeln!(writer, "{}", table)?;
        }
        Ok(())
    }
}

fn write_obj_changes<T: Display>(
    values: Vec<T>,
    output_string: &str,
    builder: &mut TableBuilder,
) -> std::fmt::Result {
    if !values.is_empty() {
        builder.push_record(vec![format!("{} Objects: ", output_string)]);
        for obj in values {
            builder.push_record(vec![format!("{}", obj)]);
        }
    }
    Ok(())
}

pub fn get_new_package_obj_from_response(
    response: &PeraTransactionBlockResponse,
) -> Option<ObjectRef> {
    response.object_changes.as_ref().and_then(|changes| {
        changes
            .iter()
            .find(|change| matches!(change, ObjectChange::Published { .. }))
            .map(|change| change.object_ref())
    })
}

pub fn get_new_package_upgrade_cap_from_response(
    response: &PeraTransactionBlockResponse,
) -> Option<ObjectRef> {
    response.object_changes.as_ref().and_then(|changes| {
        changes
            .iter()
            .find(|change| {
                matches!(change, ObjectChange::Created {
                    owner: Owner::AddressOwner(_),
                    object_type: StructTag {
                        address: PERA_FRAMEWORK_ADDRESS,
                        module,
                        name,
                        ..
                    },
                    ..
                } if module.as_str() == "package" && name.as_str() == "UpgradeCap")
            })
            .map(|change| change.object_ref())
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename = "TransactionBlockKind", tag = "kind")]
pub enum PeraTransactionBlockKind {
    /// A system transaction that will update epoch information on-chain.
    ChangeEpoch(PeraChangeEpoch),
    /// A system transaction used for initializing the initial state of the chain.
    Genesis(PeraGenesisTransaction),
    /// A system transaction marking the start of a series of transactions scheduled as part of a
    /// checkpoint
    ConsensusCommitPrologue(PeraConsensusCommitPrologue),
    /// A series of transactions where the results of one transaction can be used in future
    /// transactions
    ProgrammableTransaction(PeraProgrammableTransactionBlock),
    /// A transaction which updates global authenticator state
    AuthenticatorStateUpdate(PeraAuthenticatorStateUpdate),
    /// A transaction which updates global randomness state
    RandomnessStateUpdate(PeraRandomnessStateUpdate),
    /// The transaction which occurs only at the end of the epoch
    EndOfEpochTransaction(PeraEndOfEpochTransaction),
    ConsensusCommitPrologueV2(PeraConsensusCommitPrologueV2),
    ConsensusCommitPrologueV3(PeraConsensusCommitPrologueV3),
    // .. more transaction types go here
    SignatureMPCOutput(PeraSignatureMPCOutput),
}

impl Display for PeraTransactionBlockKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut writer = String::new();
        match &self {
            Self::ChangeEpoch(e) => {
                writeln!(writer, "Transaction Kind: Epoch Change")?;
                writeln!(writer, "New epoch ID: {}", e.epoch)?;
                writeln!(writer, "Storage gas reward: {}", e.storage_charge)?;
                writeln!(writer, "Computation gas reward: {}", e.computation_charge)?;
                writeln!(writer, "Storage rebate: {}", e.storage_rebate)?;
                writeln!(writer, "Timestamp: {}", e.epoch_start_timestamp_ms)?;
            }
            Self::Genesis(_) => {
                writeln!(writer, "Transaction Kind: Genesis Transaction")?;
            }
            Self::ConsensusCommitPrologue(p) => {
                writeln!(writer, "Transaction Kind: Consensus Commit Prologue")?;
                writeln!(
                    writer,
                    "Epoch: {}, Round: {}, Timestamp: {}",
                    p.epoch, p.round, p.commit_timestamp_ms
                )?;
            }
            Self::ConsensusCommitPrologueV2(p) => {
                writeln!(writer, "Transaction Kind: Consensus Commit Prologue V2")?;
                writeln!(
                    writer,
                    "Epoch: {}, Round: {}, Timestamp: {}, ConsensusCommitDigest: {}",
                    p.epoch, p.round, p.commit_timestamp_ms, p.consensus_commit_digest
                )?;
            }
            Self::ConsensusCommitPrologueV3(p) => {
                writeln!(writer, "Transaction Kind: Consensus Commit Prologue V3")?;
                writeln!(
                    writer,
                    "Epoch: {}, Round: {}, SubDagIndex: {:?}, Timestamp: {}, ConsensusCommitDigest: {}",
                    p.epoch, p.round, p.sub_dag_index, p.commit_timestamp_ms, p.consensus_commit_digest
                )?;
            }
            Self::ProgrammableTransaction(p) => {
                write!(writer, "Transaction Kind: Programmable")?;
                write!(writer, "{}", crate::displays::Pretty(p))?;
            }
            Self::AuthenticatorStateUpdate(_) => {
                writeln!(writer, "Transaction Kind: Authenticator State Update")?;
            }
            Self::RandomnessStateUpdate(_) => {
                writeln!(writer, "Transaction Kind: Randomness State Update")?;
            }
            Self::EndOfEpochTransaction(_) => {
                writeln!(writer, "Transaction Kind: End of Epoch Transaction")?;
            }
            Self::SignatureMPCOutput(_) => {
                writeln!(writer, "Transaction Kind: Signature MPC Output")?;
            }
        }
        write!(f, "{}", writer)
    }
}

impl PeraTransactionBlockKind {
    fn try_from(tx: TransactionKind, module_cache: &impl GetModule) -> Result<Self, anyhow::Error> {
        Ok(match tx {
            TransactionKind::ChangeEpoch(e) => Self::ChangeEpoch(e.into()),
            TransactionKind::Genesis(g) => Self::Genesis(PeraGenesisTransaction {
                objects: g.objects.iter().map(GenesisObject::id).collect(),
            }),
            TransactionKind::ConsensusCommitPrologue(p) => {
                Self::ConsensusCommitPrologue(PeraConsensusCommitPrologue {
                    epoch: p.epoch,
                    round: p.round,
                    commit_timestamp_ms: p.commit_timestamp_ms,
                })
            }
            TransactionKind::ConsensusCommitPrologueV2(p) => {
                Self::ConsensusCommitPrologueV2(PeraConsensusCommitPrologueV2 {
                    epoch: p.epoch,
                    round: p.round,
                    commit_timestamp_ms: p.commit_timestamp_ms,
                    consensus_commit_digest: p.consensus_commit_digest,
                })
            }
            TransactionKind::ConsensusCommitPrologueV3(p) => {
                Self::ConsensusCommitPrologueV3(PeraConsensusCommitPrologueV3 {
                    epoch: p.epoch,
                    round: p.round,
                    sub_dag_index: p.sub_dag_index,
                    commit_timestamp_ms: p.commit_timestamp_ms,
                    consensus_commit_digest: p.consensus_commit_digest,
                    consensus_determined_version_assignments: p
                        .consensus_determined_version_assignments,
                })
            }
            TransactionKind::ProgrammableTransaction(p) => Self::ProgrammableTransaction(
                PeraProgrammableTransactionBlock::try_from(p, module_cache)?,
            ),
            TransactionKind::AuthenticatorStateUpdate(update) => {
                Self::AuthenticatorStateUpdate(PeraAuthenticatorStateUpdate {
                    epoch: update.epoch,
                    round: update.round,
                    new_active_jwks: update
                        .new_active_jwks
                        .into_iter()
                        .map(PeraActiveJwk::from)
                        .collect(),
                })
            }
            TransactionKind::RandomnessStateUpdate(update) => {
                Self::RandomnessStateUpdate(PeraRandomnessStateUpdate {
                    epoch: update.epoch,
                    randomness_round: update.randomness_round.0,
                    random_bytes: update.random_bytes,
                })
            }
            TransactionKind::EndOfEpochTransaction(end_of_epoch_tx) => {
                Self::EndOfEpochTransaction(PeraEndOfEpochTransaction {
                    transactions: end_of_epoch_tx
                        .into_iter()
                        .map(|tx| match tx {
                            EndOfEpochTransactionKind::ChangeEpoch(e) => {
                                PeraEndOfEpochTransactionKind::ChangeEpoch(e.into())
                            }
                            EndOfEpochTransactionKind::AuthenticatorStateCreate => {
                                PeraEndOfEpochTransactionKind::AuthenticatorStateCreate
                            }
                            EndOfEpochTransactionKind::AuthenticatorStateExpire(expire) => {
                                PeraEndOfEpochTransactionKind::AuthenticatorStateExpire(
                                    PeraAuthenticatorStateExpire {
                                        min_epoch: expire.min_epoch,
                                    },
                                )
                            }
                            EndOfEpochTransactionKind::RandomnessStateCreate => {
                                PeraEndOfEpochTransactionKind::RandomnessStateCreate
                            }
                            EndOfEpochTransactionKind::DenyListStateCreate => {
                                PeraEndOfEpochTransactionKind::CoinDenyListStateCreate
                            }
                            EndOfEpochTransactionKind::BridgeStateCreate(chain_id) => {
                                PeraEndOfEpochTransactionKind::BridgeStateCreate(
                                    (*chain_id.as_bytes()).into(),
                                )
                            }
                            EndOfEpochTransactionKind::BridgeCommitteeInit(
                                bridge_shared_version,
                            ) => PeraEndOfEpochTransactionKind::BridgeCommitteeUpdate(
                                bridge_shared_version,
                            ),
                        })
                        .collect(),
                })
            }
            TransactionKind::DwalletMPCOutput(output) => {
                Self::SignatureMPCOutput(PeraSignatureMPCOutput {
                    session_id: output.session_id,
                    sender_address: output.initiating_address,
                    value: output.value,
                })
            }
        })
    }

    async fn try_from_with_package_resolver(
        tx: TransactionKind,
        package_resolver: Arc<Resolver<impl PackageStore>>,
    ) -> Result<Self, anyhow::Error> {
        Ok(match tx {
            TransactionKind::ChangeEpoch(e) => Self::ChangeEpoch(e.into()),
            TransactionKind::Genesis(g) => Self::Genesis(PeraGenesisTransaction {
                objects: g.objects.iter().map(GenesisObject::id).collect(),
            }),
            TransactionKind::ConsensusCommitPrologue(p) => {
                Self::ConsensusCommitPrologue(PeraConsensusCommitPrologue {
                    epoch: p.epoch,
                    round: p.round,
                    commit_timestamp_ms: p.commit_timestamp_ms,
                })
            }
            TransactionKind::ConsensusCommitPrologueV2(p) => {
                Self::ConsensusCommitPrologueV2(PeraConsensusCommitPrologueV2 {
                    epoch: p.epoch,
                    round: p.round,
                    commit_timestamp_ms: p.commit_timestamp_ms,
                    consensus_commit_digest: p.consensus_commit_digest,
                })
            }
            TransactionKind::ConsensusCommitPrologueV3(p) => {
                Self::ConsensusCommitPrologueV3(PeraConsensusCommitPrologueV3 {
                    epoch: p.epoch,
                    round: p.round,
                    sub_dag_index: p.sub_dag_index,
                    commit_timestamp_ms: p.commit_timestamp_ms,
                    consensus_commit_digest: p.consensus_commit_digest,
                    consensus_determined_version_assignments: p
                        .consensus_determined_version_assignments,
                })
            }
            TransactionKind::ProgrammableTransaction(p) => Self::ProgrammableTransaction(
                PeraProgrammableTransactionBlock::try_from_with_package_resolver(
                    p,
                    package_resolver,
                )
                .await?,
            ),
            TransactionKind::AuthenticatorStateUpdate(update) => {
                Self::AuthenticatorStateUpdate(PeraAuthenticatorStateUpdate {
                    epoch: update.epoch,
                    round: update.round,
                    new_active_jwks: update
                        .new_active_jwks
                        .into_iter()
                        .map(PeraActiveJwk::from)
                        .collect(),
                })
            }
            TransactionKind::RandomnessStateUpdate(update) => {
                Self::RandomnessStateUpdate(PeraRandomnessStateUpdate {
                    epoch: update.epoch,
                    randomness_round: update.randomness_round.0,
                    random_bytes: update.random_bytes,
                })
            }
            TransactionKind::EndOfEpochTransaction(end_of_epoch_tx) => {
                Self::EndOfEpochTransaction(PeraEndOfEpochTransaction {
                    transactions: end_of_epoch_tx
                        .into_iter()
                        .map(|tx| match tx {
                            EndOfEpochTransactionKind::ChangeEpoch(e) => {
                                PeraEndOfEpochTransactionKind::ChangeEpoch(e.into())
                            }
                            EndOfEpochTransactionKind::AuthenticatorStateCreate => {
                                PeraEndOfEpochTransactionKind::AuthenticatorStateCreate
                            }
                            EndOfEpochTransactionKind::AuthenticatorStateExpire(expire) => {
                                PeraEndOfEpochTransactionKind::AuthenticatorStateExpire(
                                    PeraAuthenticatorStateExpire {
                                        min_epoch: expire.min_epoch,
                                    },
                                )
                            }
                            EndOfEpochTransactionKind::RandomnessStateCreate => {
                                PeraEndOfEpochTransactionKind::RandomnessStateCreate
                            }
                            EndOfEpochTransactionKind::DenyListStateCreate => {
                                PeraEndOfEpochTransactionKind::CoinDenyListStateCreate
                            }
                            EndOfEpochTransactionKind::BridgeStateCreate(id) => {
                                PeraEndOfEpochTransactionKind::BridgeStateCreate(
                                    (*id.as_bytes()).into(),
                                )
                            }
                            EndOfEpochTransactionKind::BridgeCommitteeInit(seq) => {
                                PeraEndOfEpochTransactionKind::BridgeCommitteeUpdate(seq)
                            }
                        })
                        .collect(),
                })
            }
            TransactionKind::DwalletMPCOutput(output) => {
                Self::SignatureMPCOutput(PeraSignatureMPCOutput {
                    session_id: output.session_id,
                    sender_address: output.initiating_address,
                    value: output.value,
                })
            }
        })
    }

    pub fn transaction_count(&self) -> usize {
        match self {
            Self::ProgrammableTransaction(p) => p.commands.len(),
            _ => 1,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::ChangeEpoch(_) => "ChangeEpoch",
            Self::Genesis(_) => "Genesis",
            Self::ConsensusCommitPrologue(_) => "ConsensusCommitPrologue",
            Self::ConsensusCommitPrologueV2(_) => "ConsensusCommitPrologueV2",
            Self::ConsensusCommitPrologueV3(_) => "ConsensusCommitPrologueV3",
            Self::ProgrammableTransaction(_) => "ProgrammableTransaction",
            Self::AuthenticatorStateUpdate(_) => "AuthenticatorStateUpdate",
            Self::RandomnessStateUpdate(_) => "RandomnessStateUpdate",
            Self::EndOfEpochTransaction(_) => "EndOfEpochTransaction",
            Self::SignatureMPCOutput(_) => "SignatureMPCOutput",
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraChangeEpoch {
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: EpochId,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub storage_charge: u64,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub computation_charge: u64,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub storage_rebate: u64,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub epoch_start_timestamp_ms: u64,
}

impl From<ChangeEpoch> for PeraChangeEpoch {
    fn from(e: ChangeEpoch) -> Self {
        Self {
            epoch: e.epoch,
            storage_charge: e.storage_charge,
            computation_charge: e.computation_charge,
            storage_rebate: e.storage_rebate,
            epoch_start_timestamp_ms: e.epoch_start_timestamp_ms,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq)]
#[enum_dispatch(PeraTransactionBlockEffectsAPI)]
#[serde(
    rename = "TransactionBlockEffects",
    rename_all = "camelCase",
    tag = "messageVersion"
)]
pub enum PeraTransactionBlockEffects {
    V1(PeraTransactionBlockEffectsV1),
}

#[enum_dispatch]
pub trait PeraTransactionBlockEffectsAPI {
    fn status(&self) -> &PeraExecutionStatus;
    fn into_status(self) -> PeraExecutionStatus;
    fn shared_objects(&self) -> &[PeraObjectRef];
    fn created(&self) -> &[OwnedObjectRef];
    fn mutated(&self) -> &[OwnedObjectRef];
    fn unwrapped(&self) -> &[OwnedObjectRef];
    fn deleted(&self) -> &[PeraObjectRef];
    fn unwrapped_then_deleted(&self) -> &[PeraObjectRef];
    fn wrapped(&self) -> &[PeraObjectRef];
    fn gas_object(&self) -> &OwnedObjectRef;
    fn events_digest(&self) -> Option<&TransactionEventsDigest>;
    fn dependencies(&self) -> &[TransactionDigest];
    fn executed_epoch(&self) -> EpochId;
    fn transaction_digest(&self) -> &TransactionDigest;
    fn gas_cost_summary(&self) -> &GasCostSummary;

    /// Return an iterator of mutated objects, but excluding the gas object.
    fn mutated_excluding_gas(&self) -> Vec<OwnedObjectRef>;
    fn modified_at_versions(&self) -> Vec<(ObjectID, SequenceNumber)>;
    fn all_changed_objects(&self) -> Vec<(&OwnedObjectRef, WriteKind)>;
    fn all_deleted_objects(&self) -> Vec<(&PeraObjectRef, DeleteKind)>;
}

#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(
    rename = "TransactionBlockEffectsModifiedAtVersions",
    rename_all = "camelCase"
)]
pub struct PeraTransactionBlockEffectsModifiedAtVersions {
    object_id: ObjectID,
    #[schemars(with = "AsSequenceNumber")]
    #[serde_as(as = "AsSequenceNumber")]
    sequence_number: SequenceNumber,
}

/// The response from processing a transaction or a certified transaction
#[serde_as]
#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "TransactionBlockEffectsV1", rename_all = "camelCase")]
pub struct PeraTransactionBlockEffectsV1 {
    /// The status of the execution
    pub status: PeraExecutionStatus,
    /// The epoch when this transaction was executed.
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub executed_epoch: EpochId,
    pub gas_used: GasCostSummary,
    /// The version that every modified (mutated or deleted) object had before it was modified by
    /// this transaction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modified_at_versions: Vec<PeraTransactionBlockEffectsModifiedAtVersions>,
    /// The object references of the shared objects used in this transaction. Empty if no shared objects were used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_objects: Vec<PeraObjectRef>,
    /// The transaction digest
    pub transaction_digest: TransactionDigest,
    /// ObjectRef and owner of new objects created.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub created: Vec<OwnedObjectRef>,
    /// ObjectRef and owner of mutated objects, including gas object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutated: Vec<OwnedObjectRef>,
    /// ObjectRef and owner of objects that are unwrapped in this transaction.
    /// Unwrapped objects are objects that were wrapped into other objects in the past,
    /// and just got extracted out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unwrapped: Vec<OwnedObjectRef>,
    /// Object Refs of objects now deleted (the old refs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deleted: Vec<PeraObjectRef>,
    /// Object refs of objects previously wrapped in other objects but now deleted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unwrapped_then_deleted: Vec<PeraObjectRef>,
    /// Object refs of objects now wrapped in other objects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wrapped: Vec<PeraObjectRef>,
    /// The updated gas object reference. Have a dedicated field for convenient access.
    /// It's also included in mutated.
    pub gas_object: OwnedObjectRef,
    /// The digest of the events emitted during execution,
    /// can be None if the transaction does not emit any event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_digest: Option<TransactionEventsDigest>,
    /// The set of transaction digests this transaction depends on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<TransactionDigest>,
}

impl PeraTransactionBlockEffectsAPI for PeraTransactionBlockEffectsV1 {
    fn status(&self) -> &PeraExecutionStatus {
        &self.status
    }
    fn into_status(self) -> PeraExecutionStatus {
        self.status
    }
    fn shared_objects(&self) -> &[PeraObjectRef] {
        &self.shared_objects
    }
    fn created(&self) -> &[OwnedObjectRef] {
        &self.created
    }
    fn mutated(&self) -> &[OwnedObjectRef] {
        &self.mutated
    }
    fn unwrapped(&self) -> &[OwnedObjectRef] {
        &self.unwrapped
    }
    fn deleted(&self) -> &[PeraObjectRef] {
        &self.deleted
    }
    fn unwrapped_then_deleted(&self) -> &[PeraObjectRef] {
        &self.unwrapped_then_deleted
    }
    fn wrapped(&self) -> &[PeraObjectRef] {
        &self.wrapped
    }
    fn gas_object(&self) -> &OwnedObjectRef {
        &self.gas_object
    }
    fn events_digest(&self) -> Option<&TransactionEventsDigest> {
        self.events_digest.as_ref()
    }
    fn dependencies(&self) -> &[TransactionDigest] {
        &self.dependencies
    }

    fn executed_epoch(&self) -> EpochId {
        self.executed_epoch
    }

    fn transaction_digest(&self) -> &TransactionDigest {
        &self.transaction_digest
    }

    fn gas_cost_summary(&self) -> &GasCostSummary {
        &self.gas_used
    }

    fn mutated_excluding_gas(&self) -> Vec<OwnedObjectRef> {
        self.mutated
            .iter()
            .filter(|o| *o != &self.gas_object)
            .cloned()
            .collect()
    }

    fn modified_at_versions(&self) -> Vec<(ObjectID, SequenceNumber)> {
        self.modified_at_versions
            .iter()
            .map(|v| (v.object_id, v.sequence_number))
            .collect::<Vec<_>>()
    }

    fn all_changed_objects(&self) -> Vec<(&OwnedObjectRef, WriteKind)> {
        self.mutated
            .iter()
            .map(|owner_ref| (owner_ref, WriteKind::Mutate))
            .chain(
                self.created
                    .iter()
                    .map(|owner_ref| (owner_ref, WriteKind::Create)),
            )
            .chain(
                self.unwrapped
                    .iter()
                    .map(|owner_ref| (owner_ref, WriteKind::Unwrap)),
            )
            .collect()
    }

    fn all_deleted_objects(&self) -> Vec<(&PeraObjectRef, DeleteKind)> {
        self.deleted
            .iter()
            .map(|r| (r, DeleteKind::Normal))
            .chain(
                self.unwrapped_then_deleted
                    .iter()
                    .map(|r| (r, DeleteKind::UnwrapThenDelete)),
            )
            .chain(self.wrapped.iter().map(|r| (r, DeleteKind::Wrap)))
            .collect()
    }
}

impl PeraTransactionBlockEffects {
    #[cfg(any(feature = "test-utils", test))]
    pub fn new_for_testing(
        transaction_digest: TransactionDigest,
        status: PeraExecutionStatus,
    ) -> Self {
        Self::V1(PeraTransactionBlockEffectsV1 {
            transaction_digest,
            status,
            gas_object: OwnedObjectRef {
                owner: Owner::AddressOwner(PeraAddress::random_for_testing_only()),
                reference: pera_types::base_types::random_object_ref().into(),
            },
            executed_epoch: 0,
            modified_at_versions: vec![],
            gas_used: GasCostSummary::default(),
            shared_objects: vec![],
            created: vec![],
            mutated: vec![],
            unwrapped: vec![],
            deleted: vec![],
            unwrapped_then_deleted: vec![],
            wrapped: vec![],
            events_digest: None,
            dependencies: vec![],
        })
    }
}

impl TryFrom<TransactionEffects> for PeraTransactionBlockEffects {
    type Error = PeraError;

    fn try_from(effect: TransactionEffects) -> Result<Self, Self::Error> {
        Ok(PeraTransactionBlockEffects::V1(
            PeraTransactionBlockEffectsV1 {
                status: effect.status().clone().into(),
                executed_epoch: effect.executed_epoch(),
                modified_at_versions: effect
                    .modified_at_versions()
                    .into_iter()
                    .map(|(object_id, sequence_number)| {
                        PeraTransactionBlockEffectsModifiedAtVersions {
                            object_id,
                            sequence_number,
                        }
                    })
                    .collect(),
                gas_used: effect.gas_cost_summary().clone(),
                shared_objects: to_pera_object_ref(
                    effect
                        .input_shared_objects()
                        .into_iter()
                        .map(|kind| kind.object_ref())
                        .collect(),
                ),
                transaction_digest: *effect.transaction_digest(),
                created: to_owned_ref(effect.created()),
                mutated: to_owned_ref(effect.mutated().to_vec()),
                unwrapped: to_owned_ref(effect.unwrapped().to_vec()),
                deleted: to_pera_object_ref(effect.deleted().to_vec()),
                unwrapped_then_deleted: to_pera_object_ref(
                    effect.unwrapped_then_deleted().to_vec(),
                ),
                wrapped: to_pera_object_ref(effect.wrapped().to_vec()),
                gas_object: OwnedObjectRef {
                    owner: effect.gas_object().1,
                    reference: effect.gas_object().0.into(),
                },
                events_digest: effect.events_digest().copied(),
                dependencies: effect.dependencies().to_vec(),
            },
        ))
    }
}

fn owned_objref_string(obj: &OwnedObjectRef) -> String {
    format!(
        " ┌──\n │ ID: {} \n │ Owner: {} \n │ Version: {} \n │ Digest: {}\n └──",
        obj.reference.object_id,
        obj.owner,
        u64::from(obj.reference.version),
        obj.reference.digest
    )
}

fn objref_string(obj: &PeraObjectRef) -> String {
    format!(
        " ┌──\n │ ID: {} \n │ Version: {} \n │ Digest: {}\n └──",
        obj.object_id,
        u64::from(obj.version),
        obj.digest
    )
}

impl Display for PeraTransactionBlockEffects {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut builder = TableBuilder::default();

        builder.push_record(vec![format!("Digest: {}", self.transaction_digest())]);
        builder.push_record(vec![format!("Status: {:?}", self.status())]);
        builder.push_record(vec![format!("Executed Epoch: {}", self.executed_epoch())]);

        if !self.created().is_empty() {
            builder.push_record(vec![format!("\nCreated Objects: ")]);

            for oref in self.created() {
                builder.push_record(vec![owned_objref_string(oref)]);
            }
        }

        if !self.mutated().is_empty() {
            builder.push_record(vec![format!("Mutated Objects: ")]);
            for oref in self.mutated() {
                builder.push_record(vec![owned_objref_string(oref)]);
            }
        }

        if !self.shared_objects().is_empty() {
            builder.push_record(vec![format!("Shared Objects: ")]);
            for oref in self.shared_objects() {
                builder.push_record(vec![objref_string(oref)]);
            }
        }

        if !self.deleted().is_empty() {
            builder.push_record(vec![format!("Deleted Objects: ")]);

            for oref in self.deleted() {
                builder.push_record(vec![objref_string(oref)]);
            }
        }

        if !self.wrapped().is_empty() {
            builder.push_record(vec![format!("Wrapped Objects: ")]);

            for oref in self.wrapped() {
                builder.push_record(vec![objref_string(oref)]);
            }
        }

        if !self.unwrapped().is_empty() {
            builder.push_record(vec![format!("Unwrapped Objects: ")]);
            for oref in self.unwrapped() {
                builder.push_record(vec![owned_objref_string(oref)]);
            }
        }

        builder.push_record(vec![format!(
            "Gas Object: \n{}",
            owned_objref_string(self.gas_object())
        )]);

        let gas_cost_summary = self.gas_cost_summary();
        builder.push_record(vec![format!(
            "Gas Cost Summary:\n   \
             Storage Cost: {} NPERA\n   \
             Computation Cost: {} NPERA\n   \
             Storage Rebate: {} NPERA\n   \
             Non-refundable Storage Fee: {} NPERA",
            gas_cost_summary.storage_cost,
            gas_cost_summary.computation_cost,
            gas_cost_summary.storage_rebate,
            gas_cost_summary.non_refundable_storage_fee,
        )]);

        let dependencies = self.dependencies();
        if !dependencies.is_empty() {
            builder.push_record(vec![format!("\nTransaction Dependencies:")]);
            for dependency in dependencies {
                builder.push_record(vec![format!("   {}", dependency)]);
            }
        }

        let mut table = builder.build();
        table.with(TablePanel::header("Transaction Effects"));
        table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
            1,
            TableStyle::modern().get_horizontal(),
        )]));
        write!(f, "{}", table)
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DryRunTransactionBlockResponse {
    pub effects: PeraTransactionBlockEffects,
    pub events: PeraTransactionBlockEvents,
    pub object_changes: Vec<ObjectChange>,
    pub balance_changes: Vec<BalanceChange>,
    pub input: PeraTransactionBlockData,
}

#[derive(Eq, PartialEq, Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "TransactionBlockEvents", transparent)]
pub struct PeraTransactionBlockEvents {
    pub data: Vec<PeraEvent>,
}

impl PeraTransactionBlockEvents {
    pub fn try_from(
        events: TransactionEvents,
        tx_digest: TransactionDigest,
        timestamp_ms: Option<u64>,
        resolver: &mut dyn LayoutResolver,
    ) -> PeraResult<Self> {
        Ok(Self {
            data: events
                .data
                .into_iter()
                .enumerate()
                .map(|(seq, event)| {
                    let layout = resolver.get_annotated_layout(&event.type_)?;
                    PeraEvent::try_from(event, tx_digest, seq as u64, timestamp_ms, layout)
                })
                .collect::<Result<_, _>>()?,
        })
    }

    // TODO: this is only called from the indexer. Remove this once indexer moves to its own resolver.
    pub fn try_from_using_module_resolver(
        events: TransactionEvents,
        tx_digest: TransactionDigest,
        timestamp_ms: Option<u64>,
        resolver: &impl GetModule,
    ) -> PeraResult<Self> {
        Ok(Self {
            data: events
                .data
                .into_iter()
                .enumerate()
                .map(|(seq, event)| {
                    let layout = get_layout_from_struct_tag(event.type_.clone(), resolver)?;
                    PeraEvent::try_from(event, tx_digest, seq as u64, timestamp_ms, layout)
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Display for PeraTransactionBlockEvents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.data.is_empty() {
            writeln!(f, "╭─────────────────────────────╮")?;
            writeln!(f, "│ No transaction block events │")?;
            writeln!(f, "╰─────────────────────────────╯")
        } else {
            let mut builder = TableBuilder::default();

            for event in &self.data {
                builder.push_record(vec![format!("{}", event)]);
            }

            let mut table = builder.build();
            table.with(TablePanel::header("Transaction Block Events"));
            table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
                1,
                TableStyle::modern().get_horizontal(),
            )]));
            write!(f, "{}", table)
        }
    }
}

// TODO: this file might not be the best place for this struct.
/// Additional rguments supplied to dev inspect beyond what is allowed in today's API.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "DevInspectArgs", rename_all = "camelCase")]
pub struct DevInspectArgs {
    /// The sponsor of the gas for the transaction, might be different from the sender.
    pub gas_sponsor: Option<PeraAddress>,
    /// The gas budget for the transaction.
    pub gas_budget: Option<BigInt<u64>>,
    /// The gas objects used to pay for the transaction.
    pub gas_objects: Option<Vec<ObjectRef>>,
    /// Whether to skip transaction checks for the transaction.
    pub skip_checks: Option<bool>,
    /// Whether to return the raw transaction data and effects.
    pub show_raw_txn_data_and_effects: Option<bool>,
}

/// The response from processing a dev inspect transaction
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "DevInspectResults", rename_all = "camelCase")]
pub struct DevInspectResults {
    /// Summary of effects that likely would be generated if the transaction is actually run.
    /// Note however, that not all dev-inspect transactions are actually usable as transactions so
    /// it might not be possible actually generate these effects from a normal transaction.
    pub effects: PeraTransactionBlockEffects,
    /// Events that likely would be generated if the transaction is actually run.
    pub events: PeraTransactionBlockEvents,
    /// Execution results (including return values) from executing the transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<PeraExecutionResult>>,
    /// Execution error from executing the transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The raw transaction data that was dev inspected.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub raw_txn_data: Vec<u8>,
    /// The raw effects of the transaction that was dev inspected.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub raw_effects: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "PeraExecutionResult", rename_all = "camelCase")]
pub struct PeraExecutionResult {
    /// The value of any arguments that were mutably borrowed.
    /// Non-mut borrowed values are not included
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutable_reference_outputs: Vec<(/* argument */ PeraArgument, Vec<u8>, PeraTypeTag)>,
    /// The return values from the transaction
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub return_values: Vec<(Vec<u8>, PeraTypeTag)>,
}

type ExecutionResult = (
    /*  mutable_reference_outputs */ Vec<(Argument, Vec<u8>, TypeTag)>,
    /*  return_values */ Vec<(Vec<u8>, TypeTag)>,
);

impl DevInspectResults {
    pub fn new(
        effects: TransactionEffects,
        events: TransactionEvents,
        return_values: Result<Vec<ExecutionResult>, ExecutionError>,
        raw_txn_data: Vec<u8>,
        raw_effects: Vec<u8>,
        resolver: &mut dyn LayoutResolver,
    ) -> PeraResult<Self> {
        let tx_digest = *effects.transaction_digest();
        let mut error = None;
        let mut results = None;
        match return_values {
            Err(e) => error = Some(e.to_string()),
            Ok(srvs) => {
                results = Some(
                    srvs.into_iter()
                        .map(|srv| {
                            let (mutable_reference_outputs, return_values) = srv;
                            let mutable_reference_outputs = mutable_reference_outputs
                                .into_iter()
                                .map(|(a, bytes, tag)| (a.into(), bytes, PeraTypeTag::from(tag)))
                                .collect();
                            let return_values = return_values
                                .into_iter()
                                .map(|(bytes, tag)| (bytes, PeraTypeTag::from(tag)))
                                .collect();
                            PeraExecutionResult {
                                mutable_reference_outputs,
                                return_values,
                            }
                        })
                        .collect(),
                )
            }
        };
        Ok(Self {
            effects: effects.try_into()?,
            events: PeraTransactionBlockEvents::try_from(events, tx_digest, None, resolver)?,
            results,
            error,
            raw_txn_data,
            raw_effects,
        })
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum PeraTransactionBlockBuilderMode {
    /// Regular Pera Transactions that are committed on chain
    Commit,
    /// Simulated transaction that allows calling any Move function with
    /// arbitrary values.
    DevInspect,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "ExecutionStatus", rename_all = "camelCase", tag = "status")]
pub enum PeraExecutionStatus {
    // Gas used in the success case.
    Success,
    // Gas used in the failed case, and the error.
    Failure { error: String },
}

impl Display for PeraExecutionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure { error } => write!(f, "failure due to {error}"),
        }
    }
}

impl PeraExecutionStatus {
    pub fn is_ok(&self) -> bool {
        matches!(self, PeraExecutionStatus::Success { .. })
    }
    pub fn is_err(&self) -> bool {
        matches!(self, PeraExecutionStatus::Failure { .. })
    }
}

impl From<ExecutionStatus> for PeraExecutionStatus {
    fn from(status: ExecutionStatus) -> Self {
        match status {
            ExecutionStatus::Success => Self::Success,
            ExecutionStatus::Failure {
                error,
                command: None,
            } => Self::Failure {
                error: format!("{error:?}"),
            },
            ExecutionStatus::Failure {
                error,
                command: Some(idx),
            } => Self::Failure {
                error: format!("{error:?} in command {idx}"),
            },
        }
    }
}

fn to_pera_object_ref(refs: Vec<ObjectRef>) -> Vec<PeraObjectRef> {
    refs.into_iter().map(PeraObjectRef::from).collect()
}

fn to_owned_ref(owned_refs: Vec<(ObjectRef, Owner)>) -> Vec<OwnedObjectRef> {
    owned_refs
        .into_iter()
        .map(|(oref, owner)| OwnedObjectRef {
            owner,
            reference: oref.into(),
        })
        .collect()
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename = "GasData", rename_all = "camelCase")]
pub struct PeraGasData {
    pub payment: Vec<PeraObjectRef>,
    pub owner: PeraAddress,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub price: u64,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub budget: u64,
}

impl Display for PeraGasData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Gas Owner: {}", self.owner)?;
        writeln!(f, "Gas Budget: {} NPERA", self.budget)?;
        writeln!(f, "Gas Price: {} NPERA", self.price)?;
        writeln!(f, "Gas Payment:")?;
        for payment in &self.payment {
            write!(f, "{} ", objref_string(payment))?;
        }
        writeln!(f)
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq)]
#[enum_dispatch(PeraTransactionBlockDataAPI)]
#[serde(
    rename = "TransactionBlockData",
    rename_all = "camelCase",
    tag = "messageVersion"
)]
pub enum PeraTransactionBlockData {
    V1(PeraTransactionBlockDataV1),
}

#[enum_dispatch]
pub trait PeraTransactionBlockDataAPI {
    fn transaction(&self) -> &PeraTransactionBlockKind;
    fn sender(&self) -> &PeraAddress;
    fn gas_data(&self) -> &PeraGasData;
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename = "TransactionBlockDataV1", rename_all = "camelCase")]
pub struct PeraTransactionBlockDataV1 {
    pub transaction: PeraTransactionBlockKind,
    pub sender: PeraAddress,
    pub gas_data: PeraGasData,
}

impl PeraTransactionBlockDataAPI for PeraTransactionBlockDataV1 {
    fn transaction(&self) -> &PeraTransactionBlockKind {
        &self.transaction
    }
    fn sender(&self) -> &PeraAddress {
        &self.sender
    }
    fn gas_data(&self) -> &PeraGasData {
        &self.gas_data
    }
}

impl PeraTransactionBlockData {
    pub fn move_calls(&self) -> Vec<&PeraProgrammableMoveCall> {
        match self {
            Self::V1(data) => match &data.transaction {
                PeraTransactionBlockKind::ProgrammableTransaction(pt) => pt
                    .commands
                    .iter()
                    .filter_map(|command| match command {
                        PeraCommand::MoveCall(c) => Some(&**c),
                        _ => None,
                    })
                    .collect(),
                _ => vec![],
            },
        }
    }
}

impl Display for PeraTransactionBlockData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::V1(data) => {
                writeln!(f, "Sender: {}", data.sender)?;
                writeln!(f, "{}", self.gas_data())?;
                writeln!(f, "{}", data.transaction)
            }
        }
    }
}

impl PeraTransactionBlockData {
    pub fn try_from(
        data: TransactionData,
        module_cache: &impl GetModule,
    ) -> Result<Self, anyhow::Error> {
        let message_version = data.message_version();
        let sender = data.sender();
        let gas_data = PeraGasData {
            payment: data
                .gas()
                .iter()
                .map(|obj_ref| PeraObjectRef::from(*obj_ref))
                .collect(),
            owner: data.gas_owner(),
            price: data.gas_price(),
            budget: data.gas_budget(),
        };
        let transaction = PeraTransactionBlockKind::try_from(data.into_kind(), module_cache)?;
        match message_version {
            1 => Ok(PeraTransactionBlockData::V1(PeraTransactionBlockDataV1 {
                transaction,
                sender,
                gas_data,
            })),
            _ => Err(anyhow::anyhow!(
                "Support for TransactionData version {} not implemented",
                message_version
            )),
        }
    }

    pub async fn try_from_with_package_resolver(
        data: TransactionData,
        package_resolver: Arc<Resolver<impl PackageStore>>,
    ) -> Result<Self, anyhow::Error> {
        let message_version = data.message_version();
        let sender = data.sender();
        let gas_data = PeraGasData {
            payment: data
                .gas()
                .iter()
                .map(|obj_ref| PeraObjectRef::from(*obj_ref))
                .collect(),
            owner: data.gas_owner(),
            price: data.gas_price(),
            budget: data.gas_budget(),
        };
        let transaction = PeraTransactionBlockKind::try_from_with_package_resolver(
            data.into_kind(),
            package_resolver,
        )
        .await?;
        match message_version {
            1 => Ok(PeraTransactionBlockData::V1(PeraTransactionBlockDataV1 {
                transaction,
                sender,
                gas_data,
            })),
            _ => Err(anyhow::anyhow!(
                "Support for TransactionData version {} not implemented",
                message_version
            )),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename = "TransactionBlock", rename_all = "camelCase")]
pub struct PeraTransactionBlock {
    pub data: PeraTransactionBlockData,
    pub tx_signatures: Vec<GenericSignature>,
}

impl PeraTransactionBlock {
    pub fn try_from(
        data: SenderSignedData,
        module_cache: &impl GetModule,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            data: PeraTransactionBlockData::try_from(
                data.intent_message().value.clone(),
                module_cache,
            )?,
            tx_signatures: data.tx_signatures().to_vec(),
        })
    }

    // TODO: the PeraTransactionBlock `try_from` can be removed after cleaning up indexer v1, so are the related
    // `try_from` methods for nested structs like PeraTransactionBlockData etc.
    pub async fn try_from_with_package_resolver(
        data: SenderSignedData,
        package_resolver: Arc<Resolver<impl PackageStore>>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            data: PeraTransactionBlockData::try_from_with_package_resolver(
                data.intent_message().value.clone(),
                package_resolver,
            )
            .await?,
            tx_signatures: data.tx_signatures().to_vec(),
        })
    }
}

impl Display for PeraTransactionBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut builder = TableBuilder::default();

        builder.push_record(vec![format!("{}", self.data)]);
        builder.push_record(vec![format!("Signatures:")]);
        for tx_sig in &self.tx_signatures {
            builder.push_record(vec![format!(
                "   {}\n",
                match tx_sig {
                    Signature(sig) => Base64::from_bytes(sig.signature_bytes()).encoded(),
                    _ => Base64::from_bytes(tx_sig.as_ref()).encoded(), // the signatures for multisig and zklogin are not perated to be parsed out. they should be interpreted as a whole
                }
            )]);
        }

        let mut table = builder.build();
        table.with(TablePanel::header("Transaction Data"));
        table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
            1,
            TableStyle::modern().get_horizontal(),
        )]));
        write!(f, "{}", table)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraGenesisTransaction {
    pub objects: Vec<ObjectID>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraConsensusCommitPrologue {
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub commit_timestamp_ms: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraConsensusCommitPrologueV2 {
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub commit_timestamp_ms: u64,
    pub consensus_commit_digest: ConsensusCommitDigest,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraConsensusCommitPrologueV3 {
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,
    #[schemars(with = "Option<BigInt<u64>>")]
    #[serde_as(as = "Option<BigInt<u64>>")]
    pub sub_dag_index: Option<u64>,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub commit_timestamp_ms: u64,
    pub consensus_commit_digest: ConsensusCommitDigest,
    pub consensus_determined_version_assignments: ConsensusDeterminedVersionAssignments,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraAuthenticatorStateUpdate {
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub round: u64,

    pub new_active_jwks: Vec<PeraActiveJwk>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraRandomnessStateUpdate {
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,

    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub randomness_round: u64,
    pub random_bytes: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraEndOfEpochTransaction {
    pub transactions: Vec<PeraEndOfEpochTransactionKind>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum PeraEndOfEpochTransactionKind {
    ChangeEpoch(PeraChangeEpoch),
    AuthenticatorStateCreate,
    AuthenticatorStateExpire(PeraAuthenticatorStateExpire),
    RandomnessStateCreate,
    CoinDenyListStateCreate,
    BridgeStateCreate(CheckpointDigest),
    BridgeCommitteeUpdate(SequenceNumber),
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraSignatureMPCOutput {
    pub session_id: ObjectID,
    pub sender_address: PeraAddress,
    pub value: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraAuthenticatorStateExpire {
    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub min_epoch: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraActiveJwk {
    pub jwk_id: PeraJwkId,
    pub jwk: PeraJWK,

    #[schemars(with = "BigInt<u64>")]
    #[serde_as(as = "BigInt<u64>")]
    pub epoch: u64,
}

impl From<ActiveJwk> for PeraActiveJwk {
    fn from(active_jwk: ActiveJwk) -> Self {
        Self {
            jwk_id: PeraJwkId {
                iss: active_jwk.jwk_id.iss.clone(),
                kid: active_jwk.jwk_id.kid.clone(),
            },
            jwk: PeraJWK {
                kty: active_jwk.jwk.kty.clone(),
                e: active_jwk.jwk.e.clone(),
                n: active_jwk.jwk.n.clone(),
                alg: active_jwk.jwk.alg.clone(),
            },
            epoch: active_jwk.epoch,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraJwkId {
    pub iss: String,
    pub kid: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraJWK {
    pub kty: String,
    pub e: String,
    pub n: String,
    pub alg: String,
}

#[serde_as]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "InputObjectKind")]
pub enum PeraInputObjectKind {
    // A Move package, must be immutable.
    MovePackage(ObjectID),
    // A Move object, either immutable, or owned mutable.
    ImmOrOwnedMoveObject(PeraObjectRef),
    // A Move object that's shared and mutable.
    SharedMoveObject {
        id: ObjectID,
        #[schemars(with = "AsSequenceNumber")]
        #[serde_as(as = "AsSequenceNumber")]
        initial_shared_version: SequenceNumber,
        #[serde(default = "default_shared_object_mutability")]
        mutable: bool,
    },
}

/// A series of commands where the results of one command can be used in future
/// commands
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraProgrammableTransactionBlock {
    /// Input objects or primitive values
    pub inputs: Vec<PeraCallArg>,
    #[serde(rename = "transactions")]
    /// The transactions to be executed sequentially. A failure in any transaction will
    /// result in the failure of the entire programmable transaction block.
    pub commands: Vec<PeraCommand>,
}

impl Display for PeraProgrammableTransactionBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self { inputs, commands } = self;
        writeln!(f, "Inputs: {inputs:?}")?;
        writeln!(f, "Commands: [")?;
        for c in commands {
            writeln!(f, "  {c},")?;
        }
        writeln!(f, "]")
    }
}

impl PeraProgrammableTransactionBlock {
    fn try_from(
        value: ProgrammableTransaction,
        module_cache: &impl GetModule,
    ) -> Result<Self, anyhow::Error> {
        let ProgrammableTransaction { inputs, commands } = value;
        let input_types = Self::resolve_input_type(&inputs, &commands, module_cache);
        Ok(PeraProgrammableTransactionBlock {
            inputs: inputs
                .into_iter()
                .zip(input_types)
                .map(|(arg, layout)| PeraCallArg::try_from(arg, layout.as_ref()))
                .collect::<Result<_, _>>()?,
            commands: commands.into_iter().map(PeraCommand::from).collect(),
        })
    }

    async fn try_from_with_package_resolver(
        value: ProgrammableTransaction,
        package_resolver: Arc<Resolver<impl PackageStore>>,
    ) -> Result<Self, anyhow::Error> {
        let input_types = package_resolver.pure_input_layouts(&value).await?;
        let ProgrammableTransaction { inputs, commands } = value;
        Ok(PeraProgrammableTransactionBlock {
            inputs: inputs
                .into_iter()
                .zip(input_types)
                .map(|(arg, layout)| PeraCallArg::try_from(arg, layout.as_ref()))
                .collect::<Result<_, _>>()?,
            commands: commands.into_iter().map(PeraCommand::from).collect(),
        })
    }

    fn resolve_input_type(
        inputs: &[CallArg],
        commands: &[Command],
        module_cache: &impl GetModule,
    ) -> Vec<Option<MoveTypeLayout>> {
        let mut result_types = vec![None; inputs.len()];
        for command in commands.iter() {
            match command {
                Command::MoveCall(c) => {
                    let id = ModuleId::new(c.package.into(), c.module.clone());
                    let Some(types) =
                        get_signature_types(id, c.function.as_ident_str(), module_cache)
                    else {
                        return result_types;
                    };
                    for (arg, type_) in c.arguments.iter().zip(types) {
                        if let (&Argument::Input(i), Some(type_)) = (arg, type_) {
                            if let Some(x) = result_types.get_mut(i as usize) {
                                x.replace(type_);
                            }
                        }
                    }
                }
                Command::SplitCoins(_, amounts) => {
                    for arg in amounts {
                        if let &Argument::Input(i) = arg {
                            if let Some(x) = result_types.get_mut(i as usize) {
                                x.replace(MoveTypeLayout::U64);
                            }
                        }
                    }
                }
                Command::TransferObjects(_, Argument::Input(i)) => {
                    if let Some(x) = result_types.get_mut((*i) as usize) {
                        x.replace(MoveTypeLayout::Address);
                    }
                }
                _ => {}
            }
        }
        result_types
    }
}

fn get_signature_types(
    id: ModuleId,
    function: &IdentStr,
    module_cache: &impl GetModule,
) -> Option<Vec<Option<MoveTypeLayout>>> {
    use std::borrow::Borrow;
    if let Ok(Some(module)) = module_cache.get_module_by_id(&id) {
        let module: &CompiledModule = module.borrow();
        let func = module
            .function_handles
            .iter()
            .find(|f| module.identifier_at(f.name) == function)?;
        Some(
            module
                .signature_at(func.parameters)
                .0
                .iter()
                .map(|s| primitive_type(module, &[], s).1)
                .collect(),
        )
    } else {
        None
    }
}

/// A single transaction in a programmable transaction block.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename = "PeraTransaction")]
pub enum PeraCommand {
    /// A call to either an entry or a public Move function
    MoveCall(Box<PeraProgrammableMoveCall>),
    /// `(Vec<forall T:key+store. T>, address)`
    /// It sends n-objects to the specified address. These objects must have store
    /// (public transfer) and either the previous owner must be an address or the object must
    /// be newly created.
    TransferObjects(Vec<PeraArgument>, PeraArgument),
    /// `(&mut Coin<T>, Vec<u64>)` -> `Vec<Coin<T>>`
    /// It splits off some amounts into a new coins with those amounts
    SplitCoins(PeraArgument, Vec<PeraArgument>),
    /// `(&mut Coin<T>, Vec<Coin<T>>)`
    /// It merges n-coins into the first coin
    MergeCoins(PeraArgument, Vec<PeraArgument>),
    /// Publishes a Move package. It takes the package bytes and a list of the package's transitive
    /// dependencies to link against on-chain.
    Publish(Vec<ObjectID>),
    /// Upgrades a Move package
    Upgrade(Vec<ObjectID>, ObjectID, PeraArgument),
    /// `forall T: Vec<T> -> vector<T>`
    /// Given n-values of the same type, it constructs a vector. For non objects or an empty vector,
    /// the type tag must be specified.
    MakeMoveVec(Option<String>, Vec<PeraArgument>),
}

impl Display for PeraCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MoveCall(p) => {
                write!(f, "MoveCall({p})")
            }
            Self::MakeMoveVec(ty_opt, elems) => {
                write!(f, "MakeMoveVec(")?;
                if let Some(ty) = ty_opt {
                    write!(f, "Some{ty}")?;
                } else {
                    write!(f, "None")?;
                }
                write!(f, ",[")?;
                write_sep(f, elems, ",")?;
                write!(f, "])")
            }
            Self::TransferObjects(objs, addr) => {
                write!(f, "TransferObjects([")?;
                write_sep(f, objs, ",")?;
                write!(f, "],{addr})")
            }
            Self::SplitCoins(coin, amounts) => {
                write!(f, "SplitCoins({coin},")?;
                write_sep(f, amounts, ",")?;
                write!(f, ")")
            }
            Self::MergeCoins(target, coins) => {
                write!(f, "MergeCoins({target},")?;
                write_sep(f, coins, ",")?;
                write!(f, ")")
            }
            Self::Publish(deps) => {
                write!(f, "Publish(<modules>,")?;
                write_sep(f, deps, ",")?;
                write!(f, ")")
            }
            Self::Upgrade(deps, current_package_id, ticket) => {
                write!(f, "Upgrade(<modules>, {ticket},")?;
                write_sep(f, deps, ",")?;
                write!(f, ", {current_package_id}")?;
                write!(f, ")")
            }
        }
    }
}

impl From<Command> for PeraCommand {
    fn from(value: Command) -> Self {
        match value {
            Command::MoveCall(m) => PeraCommand::MoveCall(Box::new((*m).into())),
            Command::TransferObjects(args, arg) => PeraCommand::TransferObjects(
                args.into_iter().map(PeraArgument::from).collect(),
                arg.into(),
            ),
            Command::SplitCoins(arg, args) => PeraCommand::SplitCoins(
                arg.into(),
                args.into_iter().map(PeraArgument::from).collect(),
            ),
            Command::MergeCoins(arg, args) => PeraCommand::MergeCoins(
                arg.into(),
                args.into_iter().map(PeraArgument::from).collect(),
            ),
            Command::Publish(_modules, dep_ids) => PeraCommand::Publish(dep_ids),
            Command::MakeMoveVec(tag_opt, args) => PeraCommand::MakeMoveVec(
                tag_opt.map(|tag| tag.to_string()),
                args.into_iter().map(PeraArgument::from).collect(),
            ),
            Command::Upgrade(_modules, dep_ids, current_package_id, ticket) => {
                PeraCommand::Upgrade(dep_ids, current_package_id, PeraArgument::from(ticket))
            }
        }
    }
}

/// An argument to a transaction in a programmable transaction block
#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum PeraArgument {
    /// The gas coin. The gas coin can only be used by-ref, except for with
    /// `TransferObjects`, which can use it by-value.
    GasCoin,
    /// One of the input objects or primitive values (from
    /// `ProgrammableTransactionBlock` inputs)
    Input(u16),
    /// The result of another transaction (from `ProgrammableTransactionBlock` transactions)
    Result(u16),
    /// Like a `Result` but it accesses a nested result. Currently, the only usage
    /// of this is to access a value from a Move call with multiple return values.
    NestedResult(u16, u16),
}

impl Display for PeraArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GasCoin => write!(f, "GasCoin"),
            Self::Input(i) => write!(f, "Input({i})"),
            Self::Result(i) => write!(f, "Result({i})"),
            Self::NestedResult(i, j) => write!(f, "NestedResult({i},{j})"),
        }
    }
}

impl From<Argument> for PeraArgument {
    fn from(value: Argument) -> Self {
        match value {
            Argument::GasCoin => Self::GasCoin,
            Argument::Input(i) => Self::Input(i),
            Argument::Result(i) => Self::Result(i),
            Argument::NestedResult(i, j) => Self::NestedResult(i, j),
        }
    }
}

/// The transaction for calling a Move function, either an entry function or a public
/// function (which cannot return references).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PeraProgrammableMoveCall {
    /// The package containing the module and function.
    pub package: ObjectID,
    /// The specific module in the package containing the function.
    pub module: String,
    /// The function to be called.
    pub function: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// The type arguments to the function.
    pub type_arguments: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// The arguments to the function.
    pub arguments: Vec<PeraArgument>,
}

fn write_sep<T: Display>(
    f: &mut Formatter<'_>,
    items: impl IntoIterator<Item = T>,
    sep: &str,
) -> std::fmt::Result {
    let mut xs = items.into_iter().peekable();
    while let Some(x) = xs.next() {
        write!(f, "{x}")?;
        if xs.peek().is_some() {
            write!(f, "{sep}")?;
        }
    }
    Ok(())
}

impl Display for PeraProgrammableMoveCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            package,
            module,
            function,
            type_arguments,
            arguments,
        } = self;
        write!(f, "{package}::{module}::{function}")?;
        if !type_arguments.is_empty() {
            write!(f, "<")?;
            write_sep(f, type_arguments, ",")?;
            write!(f, ">")?;
        }
        write!(f, "(")?;
        write_sep(f, arguments, ",")?;
        write!(f, ")")
    }
}

impl From<ProgrammableMoveCall> for PeraProgrammableMoveCall {
    fn from(value: ProgrammableMoveCall) -> Self {
        let ProgrammableMoveCall {
            package,
            module,
            function,
            type_arguments,
            arguments,
        } = value;
        Self {
            package,
            module: module.to_string(),
            function: function.to_string(),
            type_arguments: type_arguments.into_iter().map(|t| t.to_string()).collect(),
            arguments: arguments.into_iter().map(PeraArgument::from).collect(),
        }
    }
}

const fn default_shared_object_mutability() -> bool {
    true
}

impl From<InputObjectKind> for PeraInputObjectKind {
    fn from(input: InputObjectKind) -> Self {
        match input {
            InputObjectKind::MovePackage(id) => Self::MovePackage(id),
            InputObjectKind::ImmOrOwnedMoveObject(oref) => Self::ImmOrOwnedMoveObject(oref.into()),
            InputObjectKind::SharedMoveObject {
                id,
                initial_shared_version,
                mutable,
            } => Self::SharedMoveObject {
                id,
                initial_shared_version,
                mutable,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename = "TypeTag", rename_all = "camelCase")]
pub struct PeraTypeTag(String);

impl PeraTypeTag {
    pub fn new(tag: String) -> Self {
        Self(tag)
    }
}

impl TryInto<TypeTag> for PeraTypeTag {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<TypeTag, Self::Error> {
        parse_pera_type_tag(&self.0)
    }
}

impl From<TypeTag> for PeraTypeTag {
    fn from(tag: TypeTag) -> Self {
        Self(format!("{}", tag))
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum RPCTransactionRequestParams {
    TransferObjectRequestParams(TransferObjectParams),
    MoveCallRequestParams(MoveCallParams),
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransferObjectParams {
    pub recipient: PeraAddress,
    pub object_id: ObjectID,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MoveCallParams {
    pub package_object_id: ObjectID,
    pub module: String,
    pub function: String,
    #[serde(default)]
    pub type_arguments: Vec<PeraTypeTag>,
    pub arguments: Vec<PeraJsonValue>,
}

#[serde_as]
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransactionBlockBytes {
    /// BCS serialized transaction data bytes without its type tag, as base-64 encoded string.
    pub tx_bytes: Base64,
    /// the gas objects to be used
    pub gas: Vec<PeraObjectRef>,
    /// objects to be used in this transaction
    pub input_objects: Vec<PeraInputObjectKind>,
}

impl TransactionBlockBytes {
    pub fn from_data(data: TransactionData) -> Result<Self, anyhow::Error> {
        Ok(Self {
            tx_bytes: Base64::from_bytes(bcs::to_bytes(&data)?.as_slice()),
            gas: data
                .gas()
                .iter()
                .map(|obj_ref| PeraObjectRef::from(*obj_ref))
                .collect(),
            input_objects: data
                .input_objects()?
                .into_iter()
                .map(PeraInputObjectKind::from)
                .collect(),
        })
    }

    pub fn to_data(self) -> Result<TransactionData, anyhow::Error> {
        bcs::from_bytes::<TransactionData>(&self.tx_bytes.to_vec().map_err(|e| anyhow::anyhow!(e))?)
            .map_err(|e| anyhow::anyhow!(e))
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "OwnedObjectRef")]
pub struct OwnedObjectRef {
    pub owner: Owner,
    pub reference: PeraObjectRef,
}

impl OwnedObjectRef {
    pub fn object_id(&self) -> ObjectID {
        self.reference.object_id
    }
    pub fn version(&self) -> SequenceNumber {
        self.reference.version
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PeraCallArg {
    // Needs to become an Object Ref or Object ID, depending on object type
    Object(PeraObjectArg),
    // pure value, bcs encoded
    Pure(PeraPureValue),
}

impl PeraCallArg {
    pub fn try_from(
        value: CallArg,
        layout: Option<&MoveTypeLayout>,
    ) -> Result<Self, anyhow::Error> {
        Ok(match value {
            CallArg::Pure(p) => PeraCallArg::Pure(PeraPureValue {
                value_type: layout.map(|l| l.into()),
                value: PeraJsonValue::from_bcs_bytes(layout, &p)?,
            }),
            CallArg::Object(ObjectArg::ImmOrOwnedObject((id, version, digest))) => {
                PeraCallArg::Object(PeraObjectArg::ImmOrOwnedObject {
                    object_id: id,
                    version,
                    digest,
                })
            }
            CallArg::Object(ObjectArg::SharedObject {
                id,
                initial_shared_version,
                mutable,
            }) => PeraCallArg::Object(PeraObjectArg::SharedObject {
                object_id: id,
                initial_shared_version,
                mutable,
            }),
            CallArg::Object(ObjectArg::Receiving((object_id, version, digest))) => {
                PeraCallArg::Object(PeraObjectArg::Receiving {
                    object_id,
                    version,
                    digest,
                })
            }
        })
    }

    pub fn pure(&self) -> Option<&PeraJsonValue> {
        match self {
            PeraCallArg::Pure(v) => Some(&v.value),
            _ => None,
        }
    }

    pub fn object(&self) -> Option<&ObjectID> {
        match self {
            PeraCallArg::Object(PeraObjectArg::SharedObject { object_id, .. })
            | PeraCallArg::Object(PeraObjectArg::ImmOrOwnedObject { object_id, .. })
            | PeraCallArg::Object(PeraObjectArg::Receiving { object_id, .. }) => Some(object_id),
            _ => None,
        }
    }
}

#[serde_as]
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PeraPureValue {
    #[schemars(with = "Option<String>")]
    #[serde_as(as = "Option<AsPeraTypeTag>")]
    value_type: Option<TypeTag>,
    value: PeraJsonValue,
}

impl PeraPureValue {
    pub fn value(&self) -> PeraJsonValue {
        self.value.clone()
    }

    pub fn value_type(&self) -> Option<TypeTag> {
        self.value_type.clone()
    }
}

#[serde_as]
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "objectType", rename_all = "camelCase")]
pub enum PeraObjectArg {
    // A Move object, either immutable, or owned mutable.
    #[serde(rename_all = "camelCase")]
    ImmOrOwnedObject {
        object_id: ObjectID,
        #[schemars(with = "AsSequenceNumber")]
        #[serde_as(as = "AsSequenceNumber")]
        version: SequenceNumber,
        digest: ObjectDigest,
    },
    // A Move object that's shared.
    // SharedObject::mutable controls whether caller asks for a mutable reference to shared object.
    #[serde(rename_all = "camelCase")]
    SharedObject {
        object_id: ObjectID,
        #[schemars(with = "AsSequenceNumber")]
        #[serde_as(as = "AsSequenceNumber")]
        initial_shared_version: SequenceNumber,
        mutable: bool,
    },
    // A reference to a Move object that's going to be received in the transaction.
    #[serde(rename_all = "camelCase")]
    Receiving {
        object_id: ObjectID,
        #[schemars(with = "AsSequenceNumber")]
        #[serde_as(as = "AsSequenceNumber")]
        version: SequenceNumber,
        digest: ObjectDigest,
    },
}

#[derive(Clone)]
pub struct EffectsWithInput {
    pub effects: PeraTransactionBlockEffects,
    pub input: TransactionData,
}

impl From<EffectsWithInput> for PeraTransactionBlockEffects {
    fn from(e: EffectsWithInput) -> Self {
        e.effects
    }
}

#[serde_as]
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub enum TransactionFilter {
    /// Query by checkpoint.
    Checkpoint(
        #[schemars(with = "BigInt<u64>")]
        #[serde_as(as = "Readable<BigInt<u64>, _>")]
        CheckpointSequenceNumber,
    ),
    /// Query by move function.
    MoveFunction {
        package: ObjectID,
        module: Option<String>,
        function: Option<String>,
    },
    /// Query by input object.
    InputObject(ObjectID),
    /// Query by changed object, including created, mutated and unwrapped objects.
    ChangedObject(ObjectID),
    /// Query by sender address.
    FromAddress(PeraAddress),
    /// Query by recipient address.
    ToAddress(PeraAddress),
    /// Query by sender and recipient address.
    FromAndToAddress { from: PeraAddress, to: PeraAddress },
    /// Query txs that have a given address as sender or recipient.
    FromOrToAddress { addr: PeraAddress },
    /// Query by transaction kind
    TransactionKind(String),
    /// Query transactions of any given kind in the input.
    TransactionKindIn(Vec<String>),
}

impl Filter<EffectsWithInput> for TransactionFilter {
    fn matches(&self, item: &EffectsWithInput) -> bool {
        let _scope = monitored_scope("TransactionFilter::matches");
        match self {
            TransactionFilter::InputObject(o) => {
                let Ok(input_objects) = item.input.input_objects() else {
                    return false;
                };
                input_objects.iter().any(|object| object.object_id() == *o)
            }
            TransactionFilter::ChangedObject(o) => item
                .effects
                .mutated()
                .iter()
                .any(|oref: &OwnedObjectRef| &oref.reference.object_id == o),
            TransactionFilter::FromAddress(a) => &item.input.sender() == a,
            TransactionFilter::ToAddress(a) => {
                let mutated: &[OwnedObjectRef] = item.effects.mutated();
                mutated.iter().chain(item.effects.unwrapped().iter()).any(|oref: &OwnedObjectRef| {
                    matches!(oref.owner, Owner::AddressOwner(owner) if owner == *a)
                })
            }
            TransactionFilter::FromAndToAddress { from, to } => {
                Self::FromAddress(*from).matches(item) && Self::ToAddress(*to).matches(item)
            }
            TransactionFilter::MoveFunction {
                package,
                module,
                function,
            } => item.input.move_calls().into_iter().any(|(p, m, f)| {
                p == package
                    && (module.is_none() || matches!(module,  Some(m2) if m2 == &m.to_string()))
                    && (function.is_none() || matches!(function, Some(f2) if f2 == &f.to_string()))
            }),
            TransactionFilter::TransactionKind(kind) => item.input.kind().to_string() == *kind,
            TransactionFilter::TransactionKindIn(kinds) => {
                kinds.contains(&item.input.kind().to_string())
            }
            // these filters are not supported, rpc will reject these filters on subscription
            TransactionFilter::Checkpoint(_) => false,
            TransactionFilter::FromOrToAddress { addr: _ } => false,
        }
    }
}
