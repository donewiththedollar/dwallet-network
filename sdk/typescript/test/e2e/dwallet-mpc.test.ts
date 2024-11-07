// Copyright (c) dWallet Labs, Inc.
// SPDX-License-Identifier: BSD-3-Clause-Clear

import { beforeAll, describe, it } from 'vitest';

import { createDWallet } from '../../src/dwallet-mpc/dkg';
import { presign } from '../../src/dwallet-mpc/presign';
import { approveAndSign } from '../../src/dwallet-mpc/sign';
import { setup, TestToolbox } from './utils/setup';

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
		const presignOutput = await presign(toolbox.keypair, toolbox.client, dwallet!.dwalletID);
		console.log(presignOutput);
	});

	it('should sign a message successfully ', async () => {
		const dwallet = await createDWallet(toolbox.keypair, toolbox.client);
		await approveAndSign(
			dwallet?.dwalletID!,
			'',
			[Uint8Array.from([1]), Uint8Array.from([2]), Uint8Array.from([3])],
			dwallet?.dwalletID!,
			'KECCAK256',
			toolbox.keypair,
			toolbox.client,
		);
	});
});
