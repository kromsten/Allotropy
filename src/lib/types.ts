import type { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { type QueryClient, type BankExtension, type DistributionExtension, type GovExtension, type StakingExtension } from '@cosmjs/stargate';
import { type WasmExtension } from '@cosmjs/cosmwasm-stargate';

export type { TokenizeShareRecord  } from "persistenceonejs/gaia/liquid/v1beta1/liquid";

export type ChainQueryClient = QueryClient & BankExtension & WasmExtension & DistributionExtension & GovExtension & StakingExtension;
export type Decimal = string;
export type Binary = string;
export type Timestamp = Uint64;
export type Uint64 = string;
export type Uint256 = string;

export interface InstantiateMsg {
	commission_rate?: Decimal | null;
	commission_recipient?: string | null;
	curve_type: CurveType;
	decimals: number;
	name: string;
	symbol: string;
	validators: string[];
    reserve_denom: string;
    reserve_decimals: number;
}

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
	client: SigningCosmWasmClient;
	config: ChainConfig;
	accounts: Account[];
}


export type ExecuteMsg =
	| { buy: { validator?: string | null } }
	| { sell: { amount: string; validator?: string | null } }
	| { transfer: { recipient: string; amount: string } }
	| { burn: { amount: string } }
	| { send: { contract: string; amount: string; msg: Binary } }
	| { increase_allowance: { spender: string; amount: string; expires?: Expiration | null } }
	| { decrease_allowance: { spender: string; amount: string; expires?: Expiration | null } }
	| { transfer_from: { owner: string; recipient: string; amount: string } }
	| { send_from: { owner: string; contract: string; amount: string; msg: Binary } }
	| { burn_from: { owner: string; amount: string } };

export type QueryMsg =
	| { curve_info: Record<string, never> }
	| { balance: { address: string } }
	| { token_info: Record<string, never> }
	| { allowance: { owner: string; spender: string } };

export type Expiration =
	| { at_height: number }
	| { at_time: Timestamp }
	| { never: Record<string, never> };


export type CurveType = 
    | { constant: { scale: number; value: string } }
    | { linear: { scale: number; slope: string } }
    | { square_root: { scale: number; slope: string } };

export interface CurveInfoResponse {
	reserve: string;
	reserve_denom: string;
	spot_price: Decimal;
	supply: string;
}

export interface TokenInfoResponse {
	decimals: number;
	name: string;
	symbol: string;
	total_supply: string;
}


export interface ChartComponentProps {
    currentSupply?: number;
    previewSupply?: number;
    isZoomed?: boolean;
    a?: number;
    b?: number;
    maxSupply?: number;
    height?: number;
}
