// Copyright (c) dWallet Labs, Inc.
// SPDX-License-Identifier: BSD-3-Clause-Clear

import { hello_wasm } from '@dwallet-network/signature-mpc-wasm';
import { beforeAll, describe, it } from 'vitest';

import {
	launchDKGSecondRound,
	launchDKGSession,
	launchProofMPSession,
} from '../../src/signature-mpc/proof';
import { setup, TestToolbox } from './utils/setup';

function numberArrayToHexString(numbers: number[]): string {
	return numbers
		.map((num) => num.toString(16).padStart(2, '0')) // Convert each number to hex and pad if needed
		.join(''); // Join the hex values into a single string
}

function hexStringToNumberArray(hexString: string): number[] {
	// Ensure the string length is even
	if (hexString.length % 2 !== 0) {
		throw new Error('Invalid hex string. Must have an even length.');
	}

	const result: number[] = [];
	for (let i = 0; i < hexString.length; i += 2) {
		// Take each pair of hex characters and convert them to a number
		const hexPair = hexString.substring(i, i + 2);
		result.push(parseInt(hexPair, 16));
	}

	return result;
}

describe('Test signature mpc', () => {
	let toolbox: TestToolbox;

	beforeAll(async () => {
		toolbox = await setup();
	});

	// it('should create proof MPC Event', async () => {
	// 	await launchProofMPSession(toolbox.keypair, toolbox.client);
	// 	console.log(toolbox.keypair.toPeraAddress());
	// });

	// it('should create dkg MPC Event', async () => {
	// 	// been generated manually
	// 	const user_second_dkg_round_input =
	// 		'IQK00DylC7XlNiIGlgVn3L+6c5Dluy1OQM+0FuLs99Y9HiEDWV2SnvEnjxCgnFMd8mqltVshy00Xs1zQ3ZgSYEZKTl4hAoZap9uY+b+iVQNbA5fOYr+fgBoGfiWNXbbYQGaWgbtYIQKlyz8LEwW5D5q3Rei1JpGtsplNGLVb5X6URAk0QDi2uSECtq5C1qeKQmUGz/OEysrqqe/bvko7sbCT9lNLMEt8ZfshA/nax+kmBtLXoWMXpi3rgnzeRRJdMrHDF/e8AmnPA9xjIQOlpF74CE2oOTRpLdGlekwBHS0Uxq0RzntLBouAffYtUiEDhU5G6/vBA6D4aUUHIInGmt1Gk8Sqe/P2tDYrlNBP3TohAw1SPoRe5+t+Y86Ux47MiXScIyZ1dl2Qz3tzPmOCUPamIQPpoLZ35ZrUAbvzSXa3cNx3XBWU3gW+RevZDhzEbxEmHyEDyINqkQ6h9dDKnx63dzXLXmZfTw6+GMz/+RdUboFFxNchAora6QWdhKVubPDiZt0e0VP5a3BmtrIZ4eVKam98OksZIQNHRTye+XDV09xCZ149x3hsjq3IcLH2OA1Fh+0XlUkuIyEDQ8Fd43HMZ20V1B1zKy0nhnDYyTMhBibdRCrjQAgZBKEhA4lfcBAMg8wsXrisZhoD5OFHBH2KM2ByOF4b8R4ceyNmIQJPZ/qkLya2hIeb6EJfVmo/kp+qY5JsB84eFqDP2491QqBxeJVYUpglYbwZf70Qoc4H+wkZNmaQnP3cO5EYWNBNc1ErMQHMcX2yevYRYivdqtNlhOc5ujQZq1qVgvTuk8RJHDiWoruBj+nJbw/P+scGoycXJibDNSlGwc7an/hyjK5iTit4d33wMGnDd8fAbgl9NWfsFY9CUZMM+15edEARrKfK7i7zgdgOSGwZGDX2An80HA6q63Iz1dI2d7vo/TLSpGTdQD8JOSykWCny5trDdQGlXCCl1c8so+ZyFJ+SCqL09r97lewWSisS/0+9qaMkwBDwlL+tVic3Wd1dt9HG5JJpNKkToC1DMs0/zJhkf+JeULt5LqNSzqSGVqws15g9Tl7aH7+mP7MOjgU/iqUjChyXdePGtmiloM/GCa6NNT1+wO1APQuClj3qGAk0T+XKZssVK5PdyTziP5+lRV5uXT2EJob5sblfeW4moD2TzrQ8aimnoCUESiVqkm8XjMGCMC0wkYZuLOFQdHCbwPu0vZyzqQrPryH5XL+NY4t7G+8U8jwQjQpMpqxIJZmZNcAfU91EfKKbyMn14y9D81VhJgpOf3QqcpBAZM9cZ3LRjfTb0uZeu0zkIJ0XMMkmPvl+3BuyOIWwoHvchvrvTXjQ6Ib/LXpn9xMfkGzNjvJZNgtrjpAkCABEi7T5KEpPYtuC3q+tqXubW88KqRnRAEByOAEAAAAAAAAAAAAAAAAAAE0AAAAAAAAAAAAAAAAAAAAdAgAAAAAAAAAAAAAAAAAABwAAAAAAAAAAAAAAAAAAAC8BAAAAAAAAAAAAAAAAAAD2AwAAAAAAAAAAAAAAAAAALwAAAAAAAAAAAAAAAAAAAAwAAAAAAAAAAAAAAAAAAADtAQAAAAAAAAAAAAAAAAAAbQAAAAAAAAAAAAAAAAAAAJEEAAAAAAAAAAAAAAAAAABmAAAAAAAAAAAAAAAAAAAANwEAAAAAAAAAAAAAAAAAAPkAAAAAAAAAAAAAAAAAAAChAQAAAAAAAAAAAAAAAAAAMwEAAAAAAAAAAAAAAAAAACECQDh9aOUKzi4COApvTWY2MjMGuH3E8Jl5woYBp83wwac=';
	//
	// });

	it('should create dkg MPC Event', async () => {
		console.log(toolbox.keypair.toPeraAddress());
		// const firstDKGOutput = await launchDKGSession(toolbox.keypair, toolbox.client);
		// const a = numberArrayToHexString(firstDKGOutput!);
		const a =
			'001df78f3b5736463644764f313d47b3c0733cb930f710346db706bcd254fc8f08cf84d4723fc4cd26966a3bcbab2c576aefbbca0348b01e54a9b4988fde14c43517d0cba537131cc097bad5fdd1d86796e9a9c144b4c89e9c705a523c25a9cf830e4e6fb83e08ed77c18024ab2660eb6d8888e0d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008bcfa77de688a288bbc23439d153759de5e3d4416d1fbf088e615bbc4075597b4b5e433bc393ceadd39126d90da86f04a914d1abe98fe0479b9322cedf84852092cfccc82df3bdc67cdb26139a45a3fedcdc8b54c6d2c43e6c86b4bf2ec16b63fb3db2f9f99b374616f50895deea4e62c68026a700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000037237c2d9edbfdeae60ec990c5afffae8239568839daab863dbab5bb64da077b74a7f3140ba1b97510246442ac3a46a2bbc09462c2ce7ba6345a07f0884bc0fb9e90ffa418f2e00480af64375ac4b24ef09d8c848c7f0641af61d0b2aafe219ac41a9b5ae4e670bbb50949c98aedcb7d660980530100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f8eb4ba8e6b747bb7180363a91ffd661ffeaae5969868d06198ad7d34d348d5dce52a6035dfc0de055178b0781a4e7de7686a48898851977228465fb9679961111d90b4e98d391d6db41ae0c12b02be2d81bab71ba18364a88d9a121acd589c372d5b6de6a86b4d45ec6c7b87b23e7048be708f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002103d6bc7b6e845778b4cc902f4b70d09d7a3c79b9a1e05c8f04679535b9ffb64d5f';
		console.log(a);
		const b = hexStringToNumberArray(a);
		let publicKeyShareAndProof = hello_wasm(Uint8Array.from(b));
		console.log(publicKeyShareAndProof);
		await launchDKGSecondRound(
			toolbox.keypair,
			toolbox.client,
			publicKeyShareAndProof,
			firstDKGOutput!,
		);
	});
});
