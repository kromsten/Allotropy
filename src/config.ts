import chainConfig from '../config/chain.json';
import contractsConfig from '../config/contracts.json';
import accountsConfig from '../config/accounts.json';
import type { ChainConfig, Account } from './types';

export const getChainConfig = (): ChainConfig => {
	return chainConfig as ChainConfig;
};

export const getContractConfig = (): any => {
	return contractsConfig;
};

export const loadTestAccounts = (): Account[] => {
	return accountsConfig as Account[];
};
