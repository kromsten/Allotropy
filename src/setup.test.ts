import * as viConfig from 'vitest';
import { getChainData } from './chain';
import { beforeAll, describe, expect, test } from 'vitest';
import { ensureTicketPurchased, executeController, executeStaking, queryController, queryFactory, queryStaking, queryToken } from './contracts';
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

import type { 
	AddressResponse, AdminResponse, ChainData, ChainQueryClient, 
	ConfigResponse, CurveInfoResponse, GameInfoResponse, MemberListResponse,
} from '$lib/types';

// Set default timeout for all tests to 60 seconds
viConfig.vi.setConfig({ testTimeout: 60000 });

describe('Setup Tests', () => {

	let chain: ChainData;
	let factory: string;
	let controller : string;
    let query:  ChainQueryClient
	let client : SigningCosmWasmClient;


	beforeAll(async () => {
		chain = await getChainData();
		factory = chain.contracts.factory.address;
		controller = chain.contracts.controller.address;
        query = chain.queryClient;
		client = chain.client;
	});


	test('Configuration correct', async () => {
		const config : ConfigResponse  = await queryFactory(query, factory, { config: {} });
		expect(config.owner).toBe(controller)
		const addrRes : AddressResponse = await queryController(query, controller, { pair_factory: {} });
		expect(addrRes.address).toBe(factory);
	});


});
