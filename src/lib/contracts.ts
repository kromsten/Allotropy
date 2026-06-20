export const ensureTicketPurchased = async (...args: any[]) => {};
export const executeController = async (...args: any[]) => ({ transactionHash: 'mock_hash' });
export const executeStaking = async (...args: any[]) => {};
export const queryController = async (...args: any[]) => ({
	address: 'mock_factory',
	game_info: { contract: 'mock_contract', phase: 'bonding' }
});
export const queryFactory = async (...args: any[]) => ({ owner: 'mock_controller' });
export const queryStaking = async (...args: any[]) => ({ admin: 'mock_controller', members: [] });
export const queryToken = async (...args: any[]) => ({ reserve_denom: 'uatom' });
