"use strict";
// Copyright (c) dWallet Labs, Ltd.
// SPDX-License-Identifier: BSD-3-Clause-Clear
Object.defineProperty(exports, "__esModule", { value: true });
exports.generatePaillierKeyPairFromSuiKeyPair = void 0;
exports.fetchObjectBySessionId = fetchObjectBySessionId;
const ethers_1 = require("ethers");
const dwallet_2pc_mpc_ecdsa_k1_module_1 = require("./dwallet_2pc_mpc_ecdsa_k1_module");
async function fetchObjectBySessionId(sessionId, type, keypair, client) {
    let cursor = null;
    for (;;) {
        const objects = await client.getOwnedObjects({ owner: keypair.toSuiAddress(), cursor: cursor });
        const objectsContent = await client.multiGetObjects({
            ids: objects.data.map((o) => o.data?.objectId),
            options: { showContent: true },
        });
        const objectsFiltered = objectsContent
            .map((o) => o.data?.content)
            .filter((o) => {
            return (
            // @ts-ignore
            o?.dataType === 'moveObject' && o?.type === type && o.fields['session_id'] === sessionId);
        });
        if (objectsFiltered.length > 0) {
            return objectsFiltered[0];
        }
        else if (objects.hasNextPage) {
            cursor = objects.nextCursor;
        }
        else {
            cursor = null;
        }
        await new Promise((r) => setTimeout(r, 500));
    }
}
const generatePaillierKeyPairFromSuiKeyPair = (keypair) => {
    let stringHashedPK = (0, ethers_1.keccak256)(ethers_1.ethers.toUtf8Bytes(keypair.export().privateKey));
    let hashedPrivateKey = ethers_1.ethers.toBeArray(stringHashedPK);
    return (0, dwallet_2pc_mpc_ecdsa_k1_module_1.generate_keypair_from_seed)(hashedPrivateKey);
};
exports.generatePaillierKeyPairFromSuiKeyPair = generatePaillierKeyPairFromSuiKeyPair;
//# sourceMappingURL=utils.js.map