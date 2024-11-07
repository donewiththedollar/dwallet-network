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
