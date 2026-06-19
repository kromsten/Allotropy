import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

export interface ChainConfig {
	denom: string;
	prefix: string;
	chain_id: string;
	rpc_endpoint: string;
	grpc_endpoint: string;
	derivation_path: string;
	gas_adjustment: number;
	gas_price: number;
}

export interface Account {
	name: string;
	address: string;
	mnemonic: string;
}

export interface ChainData {
	wallet: any;
	address: string;
	queryClient: ChainQueryClient;
	contracts: any;
	client: any;
	config: ChainConfig;
	accounts: Account[];
}

export type ChainQueryClient = any; // Simplified for the backbone

export interface ConfigResponse {
	owner: string;
}

export interface AddressResponse {
	address: string;
}

export interface GameInfoResponse {
	game_info: {
		contract: string;
		phase: string;
	} | null;
}

export interface AdminResponse {
	admin: string;
}

export interface MemberListResponse {
	members: any[];
}

export interface CurveInfoResponse {
	reserve_denom: string;
}
