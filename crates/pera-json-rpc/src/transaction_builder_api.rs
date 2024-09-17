// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: BSD-3-Clause-Clear

use std::sync::Arc;

use async_trait::async_trait;
use fastcrypto::encoding::Base64;
use jsonrpsee::core::RpcResult;
use jsonrpsee::RpcModule;
use move_core_types::language_storage::StructTag;

use pera_core::authority::AuthorityState;
use pera_json::PeraJsonValue;
use pera_json_rpc_api::{TransactionBuilderOpenRpc, TransactionBuilderServer};
use pera_json_rpc_types::{PeraObjectDataFilter, RPCTransactionRequestParams};
use pera_json_rpc_types::{
    PeraObjectDataOptions, PeraObjectResponse, PeraTransactionBlockBuilderMode, PeraTypeTag,
    TransactionBlockBytes,
};
use pera_open_rpc::Module;
use pera_transaction_builder::{DataReader, TransactionBuilder};
use pera_types::base_types::ObjectInfo;
use pera_types::base_types::{ObjectID, PeraAddress};
use pera_types::pera_serde::BigInt;

use crate::authority_state::StateRead;
use crate::PeraRpcModule;

pub struct TransactionBuilderApi(TransactionBuilder);

impl TransactionBuilderApi {
    pub fn new(state: Arc<AuthorityState>) -> Self {
        let reader = Arc::new(AuthorityStateDataReader::new(state));
        Self(TransactionBuilder::new(reader))
    }

    pub fn new_with_data_reader(data_reader: Arc<dyn DataReader + Sync + Send>) -> Self {
        Self(TransactionBuilder::new(data_reader))
    }
}

pub struct AuthorityStateDataReader(Arc<dyn StateRead>);

impl AuthorityStateDataReader {
    pub fn new(state: Arc<AuthorityState>) -> Self {
        Self(state)
    }
}

#[async_trait]
impl DataReader for AuthorityStateDataReader {
    async fn get_owned_objects(
        &self,
        address: PeraAddress,
        object_type: StructTag,
    ) -> Result<Vec<ObjectInfo>, anyhow::Error> {
        Ok(self
            .0
            // DataReader is used internally, don't need a limit
            .get_owner_objects(
                address,
                None,
                Some(PeraObjectDataFilter::StructType(object_type)),
            )?)
    }

    async fn get_object_with_options(
        &self,
        object_id: ObjectID,
        options: PeraObjectDataOptions,
    ) -> Result<PeraObjectResponse, anyhow::Error> {
        let result = self.0.get_object_read(&object_id)?;
        Ok((result, options).try_into()?)
    }

    async fn get_reference_gas_price(&self) -> Result<u64, anyhow::Error> {
        let epoch_store = self.0.load_epoch_store_one_call_per_task();
        Ok(epoch_store.reference_gas_price())
    }
}

#[async_trait]
impl TransactionBuilderServer for TransactionBuilderApi {
    async fn transfer_object(
        &self,
        signer: PeraAddress,
        object_id: ObjectID,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
        recipient: PeraAddress,
    ) -> RpcResult<TransactionBlockBytes> {
        let data = self
            .0
            .transfer_object(signer, object_id, gas, *gas_budget, recipient)
            .await?;
        Ok(TransactionBlockBytes::from_data(data)?)
    }

    async fn transfer_pera(
        &self,
        signer: PeraAddress,
        pera_object_id: ObjectID,
        gas_budget: BigInt<u64>,
        recipient: PeraAddress,
        amount: Option<BigInt<u64>>,
    ) -> RpcResult<TransactionBlockBytes> {
        let data = self
            .0
            .transfer_pera(
                signer,
                pera_object_id,
                *gas_budget,
                recipient,
                amount.map(|a| *a),
            )
            .await?;
        Ok(TransactionBlockBytes::from_data(data)?)
    }

    async fn pay(
        &self,
        signer: PeraAddress,
        input_coins: Vec<ObjectID>,
        recipients: Vec<PeraAddress>,
        amounts: Vec<BigInt<u64>>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        let data = self
            .0
            .pay(
                signer,
                input_coins,
                recipients,
                amounts.into_iter().map(|a| *a).collect(),
                gas,
                *gas_budget,
            )
            .await?;
        Ok(TransactionBlockBytes::from_data(data)?)
    }

    async fn pay_pera(
        &self,
        signer: PeraAddress,
        input_coins: Vec<ObjectID>,
        recipients: Vec<PeraAddress>,
        amounts: Vec<BigInt<u64>>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        let data = self
            .0
            .pay_pera(
                signer,
                input_coins,
                recipients,
                amounts.into_iter().map(|a| *a).collect(),
                *gas_budget,
            )
            .await?;
        Ok(TransactionBlockBytes::from_data(data)?)
    }

    async fn pay_all_pera(
        &self,
        signer: PeraAddress,
        input_coins: Vec<ObjectID>,
        recipient: PeraAddress,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        let data = self
            .0
            .pay_all_pera(signer, input_coins, recipient, *gas_budget)
            .await?;
        Ok(TransactionBlockBytes::from_data(data)?)
    }

    async fn publish(
        &self,
        sender: PeraAddress,
        compiled_modules: Vec<Base64>,
        dependencies: Vec<ObjectID>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        let compiled_modules = compiled_modules
            .into_iter()
            .map(|data| data.to_vec().map_err(|e| anyhow::anyhow!(e)))
            .collect::<Result<Vec<_>, _>>()?;
        let data = self
            .0
            .publish(sender, compiled_modules, dependencies, gas, *gas_budget)
            .await?;
        Ok(TransactionBlockBytes::from_data(data)?)
    }

    async fn split_coin(
        &self,
        signer: PeraAddress,
        coin_object_id: ObjectID,
        split_amounts: Vec<BigInt<u64>>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        let split_amounts = split_amounts.into_iter().map(|a| *a).collect();
        let data = self
            .0
            .split_coin(signer, coin_object_id, split_amounts, gas, *gas_budget)
            .await?;
        Ok(TransactionBlockBytes::from_data(data)?)
    }

    async fn split_coin_equal(
        &self,
        signer: PeraAddress,
        coin_object_id: ObjectID,
        split_count: BigInt<u64>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        let data = self
            .0
            .split_coin_equal(signer, coin_object_id, *split_count, gas, *gas_budget)
            .await?;
        Ok(TransactionBlockBytes::from_data(data)?)
    }

    async fn merge_coin(
        &self,
        signer: PeraAddress,
        primary_coin: ObjectID,
        coin_to_merge: ObjectID,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        let data = self
            .0
            .merge_coins(signer, primary_coin, coin_to_merge, gas, *gas_budget)
            .await?;
        Ok(TransactionBlockBytes::from_data(data)?)
    }

    async fn move_call(
        &self,
        signer: PeraAddress,
        package_object_id: ObjectID,
        module: String,
        function: String,
        type_arguments: Vec<PeraTypeTag>,
        rpc_arguments: Vec<PeraJsonValue>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
        _txn_builder_mode: Option<PeraTransactionBlockBuilderMode>,
    ) -> RpcResult<TransactionBlockBytes> {
        Ok(TransactionBlockBytes::from_data(
            self.0
                .move_call(
                    signer,
                    package_object_id,
                    &module,
                    &function,
                    type_arguments,
                    rpc_arguments,
                    gas,
                    *gas_budget,
                    None,
                )
                .await?,
        )?)
    }

    async fn batch_transaction(
        &self,
        signer: PeraAddress,
        params: Vec<RPCTransactionRequestParams>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
        _txn_builder_mode: Option<PeraTransactionBlockBuilderMode>,
    ) -> RpcResult<TransactionBlockBytes> {
        Ok(TransactionBlockBytes::from_data(
            self.0
                .batch_transaction(signer, params, gas, *gas_budget)
                .await?,
        )?)
    }

    async fn request_add_stake(
        &self,
        signer: PeraAddress,
        coins: Vec<ObjectID>,
        amount: Option<BigInt<u64>>,
        validator: PeraAddress,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        let amount = amount.map(|a| *a);
        Ok(TransactionBlockBytes::from_data(
            self.0
                .request_add_stake(signer, coins, amount, validator, gas, *gas_budget)
                .await?,
        )?)
    }

    async fn request_withdraw_stake(
        &self,
        signer: PeraAddress,
        staked_pera: ObjectID,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        Ok(TransactionBlockBytes::from_data(
            self.0
                .request_withdraw_stake(signer, staked_pera, gas, *gas_budget)
                .await?,
        )?)
    }
}

impl PeraRpcModule for TransactionBuilderApi {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }

    fn rpc_doc_module() -> Module {
        TransactionBuilderOpenRpc::module_doc()
    }
}
