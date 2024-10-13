// Copyright (c) dWallet Labs, Inc.
// SPDX-License-Identifier: BSD-3-Clause-Clear
import type { PeraClient } from '../client/index.js';
import type { Keypair } from '../cryptography/index.js';
import { Transaction } from '../transactions/index.js';

const packageId = '0x3';
const dWalletProofMPCModuleName = 'proof';

/**
 * Launches a proof MPC session by calling the `launch_proof_mpc_flow` function in the `proof` module.
 */
export async function launchProofMPSession(keypair: Keypair, client: PeraClient) {
	const tx = new Transaction();
	tx.moveCall({
		target: `${packageId}::${dWalletProofMPCModuleName}::launch_proof_mpc_flow`,
		arguments: [],
	});

	await client.signAndExecuteTransaction({
		signer: keypair,
		transaction: tx,
		options: {
			showEffects: true,
		},
	});
}

let dwalletModuleName = 'dWallet';

export async function launchDKGSession(keypair: Keypair, client: PeraClient) {
	const tx = new Transaction();
	tx.moveCall({
		target: `${packageId}::${dwalletModuleName}::launch_initiate_dkg_session`,
		arguments: [],
	});

	await client.signAndExecuteTransaction({
		signer: keypair,
		transaction: tx,
		options: {
			showEffects: true,
		},
	});
}
