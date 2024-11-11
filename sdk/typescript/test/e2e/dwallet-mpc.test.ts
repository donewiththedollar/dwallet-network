// Copyright (c) dWallet Labs, Inc.
// SPDX-License-Identifier: BSD-3-Clause-Clear

import { create_sign_centralized_output } from '@dwallet-network/dwallet-mpc-wasm';
import { beforeAll, describe, it } from 'vitest';

import { createDWallet } from '../../src/dwallet-mpc/dkg';
import { DKGSessionID, mockedDWallet, mockedPresign } from '../../src/dwallet-mpc/mock';
import { presign } from '../../src/dwallet-mpc/presign';
import { Hash, signMessage } from '../../src/dwallet-mpc/sign';
import { setup, TestToolbox } from './utils/setup';

function encodeBase64(bytes: Uint8Array): string {
	return btoa(String.fromCharCode(...bytes));
}

describe('Test dwallet mpc', () => {
	let toolbox: TestToolbox;

	beforeAll(async () => {
		toolbox = await setup();
	});

	it('should create DWallet', async () => {
		console.log(toolbox.keypair.toPeraAddress());
		const dwallet = await createDWallet(toolbox.keypair, toolbox.client);
		console.log(dwallet);
	});

	it('should create presign', async () => {
		console.log(toolbox.keypair.toPeraAddress());
		const dwallet = await createDWallet(toolbox.keypair, toolbox.client);
		console.log({ dwallet });
		const presignOutput = await presign(toolbox.keypair, toolbox.client, dwallet!.dwalletID);
		console.log(
			'centralizedDKGOutput',
			encodeBase64(Uint8Array.from(dwallet?.centralizedDKGOutput!)),
		);
		console.log(
			'presign first output',
			encodeBase64(Uint8Array.from(presignOutput!.encryptionOfMaskAndMaskedKeyShare)),
		);
		console.log(
			'presign second output',
			encodeBase64(Uint8Array.from(presignOutput!.noncePublicShareAndEncryptionOfMaskedNonce)),
		);
		console.log('dwallet_id', dwallet?.dwalletID);
		const res = create_sign_centralized_output(
			Uint8Array.from(dwallet?.centralizedDKGOutput!),
			Uint8Array.from(presignOutput!.encryptionOfMaskAndMaskedKeyShare),
			Uint8Array.from(presignOutput!.noncePublicShareAndEncryptionOfMaskedNonce),
			Uint8Array.from([1, 2, 3, 4, 5]),
			Hash.KECCAK256,
			dwallet?.dwalletID.slice(2)!,
		);
		console.log(res);
	});

	it('should fetch the dwallet decentralized dkg output', async () => {
		let output = await toolbox.client.getObject({
			id: '0x0c6323df7d2d073e802902d018d5dbea6229b7a6e2b90dded25b5a2c3aaaaf9c',
			options: {
				showContent: true,
			},
		});
		let dwallet = output.data?.content?.fields as {
			id: { id: string };
			dwallet_cap_id: string;
			output: number[];
		};
		console.log(encodeBase64(dwallet.output));
	});

	it('should sign a message successfully ', async () => {
		console.log(toolbox.keypair.toPeraAddress());
		const [sign_msg, centralizedOutput, fullPresigns, hash_msg] = create_sign_centralized_output(
			Uint8Array.from(mockedDWallet.centralizedDKGOutput),
			Uint8Array.from(mockedPresign.firstRoundOutput),
			Uint8Array.from(mockedPresign.secondRoundOutput),
			Uint8Array.from([1, 2, 3, 4, 5]),
			Hash.KECCAK256,
			DKGSessionID.slice(2)!,
		);

		console.log('ok');

		let res = await signMessage(
			toolbox.keypair,
			toolbox.client,
			hash_msg,
			fullPresigns,
			mockedDWallet.decentralizedDKGOutput,
			sign_msg,
		);

		console.log(res);
	});
});
