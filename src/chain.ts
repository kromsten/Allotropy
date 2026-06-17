import { createWasmAminoConverters, setupWasmExtension, SigningCosmWasmClient, wasmTypes } from '@cosmjs/cosmwasm-stargate';
import { Decimal } from '@cosmjs/math';
import { DirectSecp256k1HdWallet, type GeneratedType, Registry } from '@cosmjs/proto-signing';
import { AminoTypes, createDefaultAminoConverters, defaultRegistryTypes, QueryClient, setupAuthzExtension, setupBankExtension } from '@cosmjs/stargate';
import { Comet38Client } from '@cosmjs/tendermint-rpc';
import { getChainConfig, getContractConfig, loadTestAccounts } from './config';
import type { ChainConfig, ChainQueryClient, Account, ChainData } from './types';
import { MsgGrant } from 'cosmjs-types/cosmos/authz/v1beta1/tx';
import { ContractExecutionAuthorization as ExecAuthz } from 'cosmjs-types/cosmwasm/wasm/v1/authz';

const CONNECT_INTERVAL = 3000;
const CONNECT_MAX_ATTEMPTS = 10;

const registryTypes: [string, GeneratedType][] = [
	...defaultRegistryTypes,
	...wasmTypes,
	[MsgGrant.typeUrl, MsgGrant],
	[ExecAuthz.typeUrl, ExecAuthz],
];

const aminoTypes = new AminoTypes({
	...createDefaultAminoConverters(),
	...createWasmAminoConverters(),
});

// Cache for multiple wallets, clients, and chain data
const walletCache: Record<string, DirectSecp256k1HdWallet> = {};
const signingClientCache: Record<string, SigningCosmWasmClient> = {};

let cometClient: Comet38Client | undefined = undefined;
let queryClient: ChainQueryClient | undefined = undefined;
let chainData: ChainData | undefined = undefined;

export const getCometClient = async (
	config: ChainConfig,
	attempt: number = 1
): Promise<Comet38Client> => {
	if (!cometClient) {
		try {
			cometClient = await Comet38Client.connect(config.rpc_endpoint);
		} catch (error) {
			if (attempt >= CONNECT_MAX_ATTEMPTS) {
				throw new Error('Max connection attempts reached. Could not connect to the chain');
			}
			await new Promise(resolve => setTimeout(resolve, CONNECT_INTERVAL));
			return await getCometClient(config, attempt + 1);
		}
	}
	return cometClient;
};

export const getQueryClient = async (
	config: ChainConfig,
	attempt: number = 1
): Promise<ChainQueryClient> => {
	if (!queryClient) {
		const cometClient = await getCometClient(config, attempt);
		queryClient = QueryClient.withExtensions(
			cometClient, setupWasmExtension, setupBankExtension, setupAuthzExtension
		);
	}
	return queryClient;
};

export const getWallet = async (
	config: ChainConfig,
	accounts: Account[],
	accountIndex: number = 0,
): Promise<DirectSecp256k1HdWallet> => {
	const account = accounts[accountIndex];
	if (!account) {
		throw new Error(`Account at index ${accountIndex} not found`);
	}

	let wallet = walletCache[account.address];
	if (!wallet) {
		wallet = await DirectSecp256k1HdWallet.fromMnemonic(account.mnemonic, {
			prefix: config.prefix,
		});
		walletCache[account.address] = wallet;
	}

	const walletAccounts = await wallet.getAccounts();
	// if (!walletAccounts.some(wa => wa.address == account.address)) {
	// 	throw Error("Runtime error: Couldn't create a wallet that matches the given address")
	// }
	return wallet;
};

export const getClients = async (
	config: ChainConfig,
	wallet: DirectSecp256k1HdWallet,
	address: string,
): Promise<{ client: SigningCosmWasmClient; queryClient: ChainQueryClient }> => {

	let client = signingClientCache[address];

	if (!client || !queryClient) {
		cometClient = await getCometClient(config);
		queryClient = QueryClient.withExtensions(
			cometClient, setupWasmExtension, setupBankExtension, setupAuthzExtension
		);

		client = await SigningCosmWasmClient.createWithSigner(cometClient, wallet, {
			gasPrice: createGasPrice(config.denom, config.gas_price.toString()),
			registry: new Registry(registryTypes),
			aminoTypes
		});

		signingClientCache[address] = client;
	}

	return { client, queryClient };
};

export const getChainData = async (accountIndex: number = 0): Promise<ChainData> => {
	if (!chainData) {
		const config = getChainConfig();
		const contracts = getContractConfig();
		const accounts = loadTestAccounts();
		const wallet = await getWallet(config, accounts, accountIndex);
		const { address } = accounts[accountIndex]!;
		const { client, queryClient } = await getClients(config, wallet, address);

		chainData = {
			wallet,
			address,
			queryClient,
			contracts,
			client,
			config,
			accounts
		};

	}
	return chainData;
};

export const getAddressClient = async (
	config: ChainConfig,
	address: string,
	accounts?: Account[]
): Promise<SigningCosmWasmClient> => {
	let client = signingClientCache[address];
	if (!client) {
		let wallet = walletCache[address];
		if (!wallet) {
			accounts ??= loadTestAccounts();
			const index = accounts.findIndex(acc => acc.address === address);
			if (index == -1) {
				throw new Error(`Account with address ${address} not found in loaded accounts`);
			}
			wallet = await getWallet(config, accounts, index);
		}
		const cometClient = await getCometClient(config);
		client = await SigningCosmWasmClient.createWithSigner(cometClient, wallet, {
			gasPrice: createGasPrice(config.denom, config.gas_price.toString()),
			registry: new Registry(registryTypes),
			aminoTypes
		});
		signingClientCache[address] = client;
	}
	return client;
};

const createGasPrice = (denom: string, amount?: string) => ({
	amount: Decimal.fromUserInput(amount ?? '0.025', 100),
	denom,
});
