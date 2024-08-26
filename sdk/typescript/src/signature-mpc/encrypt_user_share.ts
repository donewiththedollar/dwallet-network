// Copyright (c) dWallet Labs, Ltd.
// SPDX-License-Identifier: BSD-3-Clause-Clear

import { verify_user_share } from '@dwallet-network/signature-mpc-wasm';
import { expect } from 'vitest';

import type { DWalletClient } from '../client/index.js';
import type { Keypair, PublicKey } from '../cryptography/index.js';
import { Ed25519PublicKey } from '../keypairs/ed25519';
import { decrypt_user_share, Dwallet, generate_proof } from './dwallet_2pc_mpc_ecdsa_k1_module.js';
import {
	getActiveEncryptionKeyObjID,
	getEncryptionKeyByObjectId,
	transferEncryptedUserShare,
} from './dwallet.js';

export const sendUserShareToSuiPubKey = async (
	client: DWalletClient,
	keypair: Keypair,
	dwallet: Dwallet,
	destinationPublicKey: PublicKey,
	activeEncryptionKeysTableID: string,
) => {
	const activeEncryptionKeyObjID = await getActiveEncryptionKeyObjID(
		client,
		destinationPublicKey.toSuiAddress(),
		activeEncryptionKeysTableID,
	);

	const recipientData = await getEncryptionKeyByObjectId(client, activeEncryptionKeyObjID);
	let isValidEncryptionKey = await destinationPublicKey.verify(
		new Uint8Array(recipientData?.encryptionKey!),
		new Uint8Array(recipientData?.signedEncryptionKey!),
	);
	if (!isValidEncryptionKey) {
		throw new Error(
			'The destination public key has not been signed by the desired destination Sui address',
		);
	}
	const encryptedUserShareAndProof = generate_proof(
		new Uint8Array(dwallet.secretKeyShare),
		recipientData?.encryptionKey!,
	);

	return await transferEncryptedUserShare(
		client,
		keypair,
		encryptedUserShareAndProof,
		activeEncryptionKeyObjID,
		dwallet,
	);
};

type EncryptedUserShare = {
	dwalletId: string;
	encryptedUserShareAndProof: number[];
	encryptionKeyObjID: string;
	signedDKGOutput: number[];
	senderPubKey: number[];
};

export const getEncryptedUserShareByObjID = async (
	client: DWalletClient,
	objID: string,
): Promise<EncryptedUserShare | null> => {
	const response = await client.getObject({
		id: objID,
		options: { showContent: true },
	});

	const objectFields =
		response.data?.content?.dataType === 'moveObject'
			? (response.data?.content?.fields as unknown as {
					dwallet_id: string;
					encrypted_secret_share_and_proof: number[];
					encryption_key_id: string;
					signed_dkg_output: number[];
					sender_pubkey: number[];
				})
			: null;

	return objectFields
		? {
				dwalletId: objectFields.dwallet_id,
				encryptedUserShareAndProof: objectFields.encrypted_secret_share_and_proof,
				encryptionKeyObjID: objectFields.encryption_key_id,
				signedDKGOutput: objectFields.signed_dkg_output,
				senderPubKey: objectFields.sender_pubkey,
			}
		: null;
};

export const verifyEncryptedSecretShare = async (
	encryptedUserShare: EncryptedUserShare,
	expectedSourceSuiAddress: string,
	encryptionKey: Uint8Array,
	decryptionKey: Uint8Array,
	dkgOutput: Uint8Array,
): Promise<boolean> => {
	let publicKey = new Ed25519PublicKey(encryptedUserShare?.senderPubKey!);
	if (
		!(await publicKey.verify(
			new Uint8Array(dkgOutput),
			new Uint8Array(encryptedUserShare?.signedDKGOutput!),
		))
	) {
		return false;
	}
	if (publicKey.toSuiAddress() !== expectedSourceSuiAddress) {
		return false;
	}

	const decryptedKeyShare = decrypt_user_share(
		encryptionKey,
		decryptionKey,
		new Uint8Array(encryptedUserShare?.encryptedUserShareAndProof!),
	);

	return verify_user_share(
		// Take the first 32 bytes, the only ones that are non-zero, and reverse them to convert them
		// from little-endian encoding to big-endian.
		// This is because of BCS and PlaintextSpaceGroupElement serialization.
		// PlaintextSpaceGroupElement is U2048 and has 32LIMBS of 64 bits each.
		new Uint8Array(decryptedKeyShare.slice(0, 32).reverse()),
		new Uint8Array(encryptedUserShare?.dwalletDKGOutput!),
	);
};
