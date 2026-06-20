import * as viConfig from 'vitest';
import { getChainData } from '../lib/chain';
import { beforeAll, describe, expect, test } from 'vitest';
import {
	ensureTicketPurchased,
	executeController,
	executeStaking,
	queryController,
	queryFactory,
	queryStaking,
	queryToken
} from '../lib/contracts';
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

import type {
	AddressResponse,
	AdminResponse,
	ChainData,
	ChainQueryClient,
	ConfigResponse,
	CurveInfoResponse,
	GameInfoResponse,
	MemberListResponse
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
