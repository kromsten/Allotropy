import { describe, expect, test, beforeAll } from 'vitest';
import { getChainData } from '../src/lib/chain';
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { type Coin, coin } from '@cosmjs/proto-signing';
import type { ChainData, ChainQueryClient } from '$types';
import type { InstantiateMsg, ExecuteMsg } from '$types';
import { queryLiquidRecords } from '$lib/cosmos';

describe('CW20 Liquid Bond Integration', () => {
	let chain: ChainData;
    let queryClient : ChainQueryClient;
	let client: SigningCosmWasmClient;
    const CODE_ID = 1;
    let contractAddress: string;

	beforeAll(async () => {
		chain = await getChainData();
		client = chain.client;
        queryClient = chain.queryClient;
	});

	test('Instantiate contract', async () => {
		const instantiateMsg: InstantiateMsg = {
			name: 'Liquid Bond Token',
			symbol: 'LBT',
			decimals: 6,
			validators: ['val1', 'val2'],
			curve_type: { constant: { scale: 6, value: '1000000' } },
            commission_rate: '0',
            commission_recipient: chain.address,
            reserve_denom: 'uatom',
            reserve_decimals: 6
		};

		const result = await client.instantiate(
			chain.address,
			CODE_ID,
			instantiateMsg,
			'LBT Contract',
			{ amount: [{ amount: '500000', denom: 'uatom' }], gas: '500000' }
		);

        contractAddress = result.contractAddress;
		expect(contractAddress).toBeDefined();
        console.log('Contract instantiated at:', contractAddress);
	});

    test('Buy and Stake', { timeout: 60000 }, async () => {
        expect(contractAddress).toBeDefined();

       // const res = await queryLiquidRecords(queryClient, chain.address);

        // queryClient.distribution
        const vals = await queryClient.staking.validators('');
        const validator = vals.validators[0].operatorAddress;

        const buyMsg: ExecuteMsg = { buy: { validator } };
        const funds: Coin[] = [coin(1000, 'uatom')];

        const result = await client.execute(
            chain.address,
            contractAddress,
            buyMsg,
            "auto",
            'Buy tokens',
            funds
        );

        expect(result.transactionHash).toBeDefined();
        console.log('Buy transaction hash:', result.transactionHash);
        // Wait for tx to be processed
        await new Promise((r) => setTimeout(r, 15000));

        const delegation = await queryClient.staking.delegation(contractAddress, validator);
        expect(delegation.delegationResponse).toBeDefined();
        console.log('Delegation amount:', delegation.delegationResponse?.delegation?.shares);

        const rewards = await queryClient.distribution.delegationTotalRewards(contractAddress);
        expect(rewards.rewards).toBeDefined();
        // check if rewards list is not empty
        expect(rewards.rewards.length).toBeGreaterThan(0);
        console.log('Rewards found:', rewards.rewards);

        // Sell/Burn on the curve
        const sellMsg: ExecuteMsg = { sell: { amount: '900' } }; 
        const sellResult = await client.execute(
            chain.address,
            contractAddress,
            sellMsg,
            "auto",
            'Burn tokens'
        );
        expect(sellResult.transactionHash).toBeDefined();
        console.log('Sell transaction hash:', sellResult.transactionHash);

        // Wait for tx to process
        await new Promise((r) => setTimeout(r, 15000));

        const liquidRecords = await queryLiquidRecords(queryClient, chain.address);
        expect(liquidRecords).toBeDefined();
        console.log('Liquid records:', liquidRecords);
    });
});
