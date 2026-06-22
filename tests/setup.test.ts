import * as viConfig from 'vitest';
import { getChainData } from '../src/lib/chain';
import { beforeAll, describe, expect, test } from 'vitest';
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

import type {
	ChainData,
	ChainQueryClient,
} from '$types';

describe('Setup Tests', () => {
	let query: ChainQueryClient;
	let client: SigningCosmWasmClient;
	let chain: ChainData;

	let factory: string;
	let controller: string;

	beforeAll(async () => {
		chain = await getChainData();
		//console.log('Chain data loaded:', chain);
		query = chain.queryClient;
		client = chain.client;
		//factory = chain.contracts.factory.address;
		//controller = chain.contracts.controller.address;
	});

	/* 	test('Configuration correct', async () => {
		const config : ConfigResponse  = await queryFactory(query, factory, { config: {} });
		expect(config.owner).toBe(controller)
		const addrRes : AddressResponse = await queryController(query, controller, { pair_factory: {} });
		expect(addrRes.address).toBe(factory);
	}); */
});
