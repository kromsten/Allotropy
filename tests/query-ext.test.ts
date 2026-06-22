import { describe, expect, test, beforeAll } from 'vitest';
import { getChainData } from '../src/lib/chain';
import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import type { ChainData } from '$types';

describe('Chain Query Extensions', () => {
    let chain: ChainData;

    beforeAll(async () => {
        chain = await getChainData();
    });

    test('Query governance and distribution params', async () => {
        const query = chain.queryClient;
        
        // Distribution
        const distrParams = await query.distribution.params();
        expect(distrParams).toBeDefined();

        // Gov
        const govParams = await query.gov.params('voting');
        expect(govParams).toBeDefined();
        
        console.log('Query client extensions verified');
    });
});
