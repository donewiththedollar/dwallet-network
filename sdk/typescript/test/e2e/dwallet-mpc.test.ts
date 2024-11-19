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
			'DecentralizedDKGOutput',
			encodeBase64(Uint8Array.from(dwallet?.decentralizedDKGOutput!)),
		);
		console.log(
			'presign first output',
			encodeBase64(Uint8Array.from(presignOutput!.encryptionOfMaskAndMaskedKeyShare)),
		);
		console.log(
			'presign second output',
			encodeBase64(Uint8Array.from(presignOutput!.noncePublicShareAndEncryptionOfMaskedNonce)),
		);
		console.log('presign first round session id', presignOutput!.presignFirstRoundSessionId);
		console.log('dwallet_id', dwallet?.dwalletID);
		const [sign_msg, centralizedOutput, fullPresigns, hash_msg] = create_sign_centralized_output(
			Uint8Array.from(dwallet?.centralizedDKGOutput!),
			Uint8Array.from(presignOutput!.encryptionOfMaskAndMaskedKeyShare),
			Uint8Array.from(presignOutput!.noncePublicShareAndEncryptionOfMaskedNonce),
			Uint8Array.from([1, 2, 3, 4, 5]),
			Hash.KECCAK256,
			dwallet?.dwalletID.slice(2)!,
		);
		let res = await signMessage(
			toolbox.keypair,
			toolbox.client,
			hash_msg,
			fullPresigns,
			// mockedDWallet.decentralizedDKGOutput,
			Uint8Array.from(dwallet?.decentralizedDKGOutput!),
			sign_msg,
			presignOutput?.presignFirstRoundSessionId!,
		);

		console.log(res);
	});

	it('should fetch the dwallet decentralized dkg output', async () => {
		let output = await toolbox.client.getObject({
			id: '0x11e7d95fb66af2613241d313415ad2b4c9fb2dd66f484f5826149a32e74a8eb4',
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
			Hash.SHA256,
			mockedPresign.firstRoundSessionID.slice(2)!,
		);

		console.log('ok');

		let res = await signMessage(
			toolbox.keypair,
			toolbox.client,
			hash_msg,
			fullPresigns,
			mockedDWallet.decentralizedDKGOutput,
			sign_msg,
			mockedPresign.firstRoundSessionID,
		);

		console.log(res);
	});
});
