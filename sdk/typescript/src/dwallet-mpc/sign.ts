import { bcs } from '../bcs/index.js';
import type { PeraClient } from '../client/index.js';
import type { Keypair } from '../cryptography/index.js';
import { Transaction } from '../transactions/index.js';
import { dWalletModuleName, packageId } from './globals.js';

export async function approveAndSign(
	dwalletCapId: string,
	signMessagesId: string,
	messages: Uint8Array[],
	dwalletID: string,
	hash: 'KECCAK256' | 'SHA256',
	keypair: Keypair,
	client: PeraClient,
) {
	const tx = new Transaction();
	const [messageApprovals] = tx.moveCall({
		target: `${packageId}::${dWalletModuleName}::approve_messages`,
		arguments: [
			tx.object(dwalletCapId),
			tx.pure(bcs.vector(bcs.vector(bcs.u8())).serialize(messages)),
		],
	});
}

export enum Hash {
	KECCAK256 = 0,
	SHA256 = 1,
}

export async function sign(
	keypair: Keypair,
	client: PeraClient,
	hashedMessage: Uint8Array,
	presign: Uint8Array,
	dkgOutput: Uint8Array,
	centralizedSignedMessage: Uint8Array,
) {
	const tx = new Transaction();
	tx.moveCall({
		target: `${packageId}::${dWalletModuleName}::sign`,
		arguments: [
			tx.pure(bcs.vector(bcs.u8()).serialize(hashedMessage)),
			tx.pure(bcs.vector(bcs.u8()).serialize(presign)),
			tx.pure(bcs.vector(bcs.u8()).serialize(dkgOutput)),
			tx.pure(bcs.vector(bcs.u8()).serialize(centralizedSignedMessage)),
		],
	});

	const result = await client.signAndExecuteTransaction({
		signer: keypair,
		transaction: tx,
		options: {
			showEffects: true,
		},
	});

	return result.effects?.created?.[0].reference!;
}
