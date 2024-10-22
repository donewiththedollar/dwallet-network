"use strict";
// Copyright (c) dWallet Labs, Ltd.
// SPDX-License-Identifier: BSD-3-Clause-Clear
Object.defineProperty(exports, "__esModule", { value: true });
exports.transferEncryptedUserShare = exports.createActiveEncryptionKeysTable = exports.setActiveEncryptionKey = exports.getActiveEncryptionKeyObjID = exports.getEncryptionKeyByObjectId = exports.storeEncryptionKey = exports.EncryptionKeyScheme = void 0;
exports.approveAndSign = approveAndSign;
const index_js_1 = require("../bcs/index.js");
const index_js_2 = require("../builder/index.js");
const packageId = '0x3';
const dWalletModuleName = 'dwallet';
const dWallet2PCMPCECDSAK1ModuleName = 'dwallet_2pc_mpc_ecdsa_k1';
var EncryptionKeyScheme;
(function (EncryptionKeyScheme) {
    EncryptionKeyScheme[EncryptionKeyScheme["Paillier"] = 0] = "Paillier";
})(EncryptionKeyScheme || (exports.EncryptionKeyScheme = EncryptionKeyScheme = {}));
async function approveAndSign(dwalletCapId, signMessagesId, messages, keypair, client) {
    const tx = new index_js_2.TransactionBlock();
    const [messageApprovals] = tx.moveCall({
        target: `${packageId}::${dWalletModuleName}::approve_messages`,
        arguments: [
            tx.object(dwalletCapId),
            tx.pure(index_js_1.bcs.vector(index_js_1.bcs.vector(index_js_1.bcs.u8())).serialize(messages)),
        ],
    });
    tx.moveCall({
        target: `${packageId}::${dWalletModuleName}::sign`,
        typeArguments: [
            `${packageId}::${dWallet2PCMPCECDSAK1ModuleName}::SignData`,
            `${packageId}::${dWallet2PCMPCECDSAK1ModuleName}::NewSignDataEvent`,
        ],
        arguments: [tx.object(signMessagesId), messageApprovals],
    });
    await client.signAndExecuteTransactionBlock({
        signer: keypair,
        transactionBlock: tx,
        options: {
            showEffects: true,
        },
    });
    return await waitForSignOutput(client);
}
const waitForSignOutput = async (client) => {
    return new Promise((resolve) => {
        client.subscribeEvent({
            filter: {
                MoveEventType: `${packageId}::${dWalletModuleName}::SignOutputEvent`,
            },
            onMessage: (event) => {
                let eventData = event?.parsedJson;
                resolve(eventData.signatures);
            },
        });
    });
};
const storeEncryptionKey = async (encryptionKey, encryptionKeyScheme, keypair, client) => {
    let signedEncryptionKey = await keypair.sign(new Uint8Array(encryptionKey));
    const tx = new index_js_2.TransactionBlock();
    let purePubKey = tx.pure(index_js_1.bcs.vector(index_js_1.bcs.u8()).serialize(encryptionKey));
    let pureSignedPubKey = tx.pure(index_js_1.bcs.vector(index_js_1.bcs.u8()).serialize(signedEncryptionKey));
    let pureSuiPubKey = tx.pure(index_js_1.bcs.vector(index_js_1.bcs.u8()).serialize(keypair.getPublicKey().toRawBytes()));
    tx.moveCall({
        target: `${packageId}::${dWalletModuleName}::register_encryption_key`,
        arguments: [
            purePubKey,
            pureSignedPubKey,
            pureSuiPubKey,
            tx.pure(index_js_1.bcs.u8().serialize(encryptionKeyScheme)),
        ],
    });
    let result = await client.signAndExecuteTransactionBlock({
        signer: keypair,
        transactionBlock: tx,
        options: {
            showEffects: true,
        },
    });
    return result.effects?.created?.filter((o) => o.owner === 'Immutable')[0].reference;
};
exports.storeEncryptionKey = storeEncryptionKey;
const getEncryptionKeyByObjectId = async (client, encryptionKeyObjID) => {
    const response = await client.getObject({
        id: encryptionKeyObjID,
        options: { showContent: true },
    });
    const objectFields = response.data?.content?.dataType === 'moveObject'
        ? response.data?.content?.fields
        : null;
    return objectFields
        ? {
            encryptionKey: objectFields?.encryption_key,
            signedEncryptionKey: objectFields?.encryption_key_signature,
            keyOwnerAddress: objectFields?.key_owner_address,
        }
        : null;
};
exports.getEncryptionKeyByObjectId = getEncryptionKeyByObjectId;
const getActiveEncryptionKeyObjID = async (client, keyOwnerAddress, encryptionKeysHolderID) => {
    const tx = new index_js_2.TransactionBlock();
    const encryptionKeysHolder = tx.object(encryptionKeysHolderID);
    console.log(keyOwnerAddress);
    tx.moveCall({
        target: `${packageId}::${dWalletModuleName}::get_active_encryption_key`,
        arguments: [encryptionKeysHolder, tx.pure(keyOwnerAddress)],
    });
    let res = await client.devInspectTransactionBlock({
        sender: keyOwnerAddress,
        transactionBlock: tx,
    });
    return Buffer.from(new Uint8Array(res.results?.at(0)?.returnValues?.at(0)?.at(0))).toString('hex');
};
exports.getActiveEncryptionKeyObjID = getActiveEncryptionKeyObjID;
const setActiveEncryptionKey = async (client, keypair, encryptionKeyObjID, encryptionKeysHolderID) => {
    const tx = new index_js_2.TransactionBlock();
    const EncKeyObj = tx.object(encryptionKeyObjID);
    const encryptionKeysHolder = tx.object(encryptionKeysHolderID);
    tx.moveCall({
        target: `${packageId}::${dWalletModuleName}::set_active_encryption_key`,
        arguments: [encryptionKeysHolder, EncKeyObj],
    });
    return await client.signAndExecuteTransactionBlock({
        signer: keypair,
        transactionBlock: tx,
        options: {
            showEffects: true,
        },
    });
};
exports.setActiveEncryptionKey = setActiveEncryptionKey;
const createActiveEncryptionKeysTable = async (client, keypair) => {
    const tx = new index_js_2.TransactionBlock();
    tx.moveCall({
        target: `${packageId}::${dWalletModuleName}::create_active_encryption_keys`,
        arguments: [],
    });
    let result = await client.signAndExecuteTransactionBlock({
        signer: keypair,
        transactionBlock: tx,
        options: {
            showEffects: true,
        },
    });
    return result.effects?.created?.filter((o) => typeof o.owner === 'object' &&
        'Shared' in o.owner &&
        o.owner.Shared.initial_shared_version !== undefined)[0].reference;
};
exports.createActiveEncryptionKeysTable = createActiveEncryptionKeysTable;
const transferEncryptedUserShare = async (client, keypair, encryptedUserShareAndProof, encryptionKeyObjID, dwalletID) => {
    const tx = new index_js_2.TransactionBlock();
    const encryptionKey = tx.object(encryptionKeyObjID);
    const dwallet = tx.object(dwalletID);
    tx.moveCall({
        target: `${packageId}::${dWallet2PCMPCECDSAK1ModuleName}::encrypt_user_share`,
        typeArguments: [],
        arguments: [dwallet, encryptionKey, tx.pure(encryptedUserShareAndProof)],
    });
    return await client.signAndExecuteTransactionBlock({
        signer: keypair,
        transactionBlock: tx,
        options: {
            showEffects: true,
        },
    });
};
exports.transferEncryptedUserShare = transferEncryptedUserShare;
//# sourceMappingURL=dwallet.js.map