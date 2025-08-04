import type { CONTRACT_CLASS, FELT, ADDRESS, SIGNATURE } from '../api/components.js';
import type { ChainId } from '../api/index.js';
/**
 * Account Address
 */
export type Address = ADDRESS;
export type Signature = SIGNATURE;
/**
 * The transaction hash, as assigned in Starknet
 */
export type PADDED_TXN_HASH = PADDED_FELT;
/**
 * A padded felt represent 0x0 + (0-7) + (62 hex digits)
 * @pattern ^0x(0[0-7]{1}[a-fA-F0-9]{62}$)
 */
export type PADDED_FELT = string;
/**
 * A Starknet RPC spec version, only two numbers are provided
 * @pattern ^[0-9]+\\.[0-9]+$
 */
export type SpecVersion = string;
/**
 * ERC20 Token Symbol (min:1 char - max:6 chars)
 * @pattern ^[A-Za-z0-9]{1,6}$
 */
export type TokenSymbol = string;
/**
 * Starknet Token
 * Details of an onchain Starknet ERC20 token
 */
export type Asset = {
    type: 'ERC20';
    options: {
        address: Address;
        symbol?: TokenSymbol;
        decimals?: number;
        image?: string;
        name?: string;
    };
};
export type StarknetChain = {
    id: string;
    chain_id: ChainId;
    chain_name: string;
    rpc_urls?: string[];
    block_explorer_url?: string[];
    native_currency?: Asset;
    icon_urls?: string[];
};
export type Call = {
    contract_address: Address;
    entry_point: string;
    calldata?: FELT[];
};
/**
 * INVOKE_TXN_V1
 * @see https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_api_openrpc.json
 */
export interface AddInvokeTransactionParameters {
    /**
     * Calls to invoke by the account
     */
    calls: Call[];
}
export interface AddInvokeTransactionResult {
    /**
     * The hash of the invoke transaction
     */
    transaction_hash: PADDED_TXN_HASH;
}
/**
 * SPEC: DECLARE_TXN
 */
export interface AddDeclareTransactionParameters {
    compiled_class_hash: FELT;
    class_hash?: FELT;
    contract_class: CONTRACT_CLASS;
}
export interface AddDeclareTransactionResult {
    /**
     * The hash of the declare transaction
     */
    transaction_hash: PADDED_TXN_HASH;
    /**
     * The hash of the declared class
     */
    class_hash: PADDED_FELT;
}
/**
 * EIP-1102:
 * @see https://eips.ethereum.org/EIPS/eip-1102
 */
export interface RequestAccountsParameters {
    /**
     * If true, the wallet will not show the wallet-unlock UI in case of a locked wallet,
     * nor the dApp-approve UI in case of a non-allowed dApp.
     */
    silent_mode?: boolean;
}
/**
 * EIP-747:
 * @see https://github.com/ethereum/EIPs/blob/master/EIPS/eip-747.md
 */
export interface WatchAssetParameters extends Asset {
}
/**
 * EIP-3085:
 * @see https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3085.md
 */
export interface AddStarknetChainParameters extends StarknetChain {
}
export interface SwitchStarknetChainParameters {
    chainId: ChainId;
}
/**
 * SPEC: ACCOUNT_DEPLOYMENT_DATA
 */
export interface AccountDeploymentData {
    address: Address;
    class_hash: FELT;
    salt: FELT;
    calldata: FELT[];
    sigdata?: FELT[];
    version: 0 | 1;
}
export type API_VERSION = string;
/**
 * The version of wallet API the request expecting. If not specified, the latest is assumed
 */
export interface ApiVersionRequest {
    api_version?: API_VERSION;
}
//# sourceMappingURL=components.d.ts.map