import { Permission } from './constants.js';
import type { TypedData } from './typedData.js';
import * as Errors from './errors.js';
import type { API_VERSION, AccountDeploymentData, AddDeclareTransactionParameters, AddDeclareTransactionResult, AddInvokeTransactionParameters, AddInvokeTransactionResult, AddStarknetChainParameters, Address, ApiVersionRequest, RequestAccountsParameters, Signature, SpecVersion, SwitchStarknetChainParameters, WatchAssetParameters } from './components.js';
import type { ChainId } from '../api/index.js';
/**
 * Maps each RPC message type to its corresponding parameters and result type.
 */
export interface RpcTypeToMessageMap {
    /**
     * Get permissions from the wallet.
     * @returns An array of permissions.
     */
    wallet_getPermissions: {
        params?: ApiVersionRequest;
        result: Permission[] | [];
        errors: Errors.API_VERSION_NOT_SUPPORTED | Errors.UNKNOWN_ERROR;
    };
    /**
     * Request active accounts from the wallet.
     * @param params Optional parameters for requesting accounts.
     * @returns An array of account addresses as strings.
     */
    wallet_requestAccounts: {
        params?: RequestAccountsParameters & ApiVersionRequest;
        result: Address[];
        errors: Errors.API_VERSION_NOT_SUPPORTED | Errors.UNKNOWN_ERROR;
    };
    /**
     * Watch an asset in the wallet.
     * @param params The parameters required to watch an asset.
     * @returns A boolean indicating if the operation was successful.
     */
    wallet_watchAsset: {
        params: WatchAssetParameters & ApiVersionRequest;
        result: boolean;
        errors: Errors.NOT_ERC20 | Errors.INVALID_REQUEST_PAYLOAD | Errors.USER_REFUSED_OP | Errors.API_VERSION_NOT_SUPPORTED | Errors.UNKNOWN_ERROR;
    };
    /**
     * Add a new Starknet chain to the wallet.
     * @param params The parameters required to add a new chain.
     * @returns A boolean indicating if the operation was successful.
     */
    wallet_addStarknetChain: {
        params: AddStarknetChainParameters & ApiVersionRequest;
        result: boolean;
        errors: Errors.INVALID_REQUEST_PAYLOAD | Errors.USER_REFUSED_OP | Errors.API_VERSION_NOT_SUPPORTED | Errors.UNKNOWN_ERROR;
    };
    /**
     * Switch the current Starknet chain in the wallet.
     * @param params The parameters required to switch chains.
     * @returns A boolean indicating if the operation was successful.
     */
    wallet_switchStarknetChain: {
        params: SwitchStarknetChainParameters & ApiVersionRequest;
        result: boolean;
        errors: Errors.UNLISTED_NETWORK | Errors.USER_REFUSED_OP | Errors.API_VERSION_NOT_SUPPORTED | Errors.UNKNOWN_ERROR;
    };
    /**
     * Request the current chain ID from the wallet.
     * @returns The current Starknet chain ID.
     */
    wallet_requestChainId: {
        params?: ApiVersionRequest;
        result: ChainId;
        errors: Errors.API_VERSION_NOT_SUPPORTED | Errors.UNKNOWN_ERROR;
    };
    /**
     * Get deployment data for a contract.
     * @returns The deployment data result.
     */
    wallet_deploymentData: {
        params?: ApiVersionRequest;
        result: AccountDeploymentData;
        errors: Errors.ACCOUNT_ALREADY_DEPLOYED | Errors.API_VERSION_NOT_SUPPORTED | Errors.UNKNOWN_ERROR;
    };
    /**
     * Add an invoke transaction to the wallet.
     * @param params The parameters required for the invoke transaction.
     * @returns The result of adding the invoke transaction.
     */
    wallet_addInvokeTransaction: {
        params: AddInvokeTransactionParameters & ApiVersionRequest;
        result: AddInvokeTransactionResult;
        errors: Errors.INVALID_REQUEST_PAYLOAD | Errors.USER_REFUSED_OP | Errors.API_VERSION_NOT_SUPPORTED | Errors.UNKNOWN_ERROR;
    };
    /**
     * Add a declare transaction to the wallet.
     * @param params The parameters required for the declare transaction.
     * @returns The result of adding the declare transaction.
     */
    wallet_addDeclareTransaction: {
        params: AddDeclareTransactionParameters & ApiVersionRequest;
        result: AddDeclareTransactionResult;
        errors: Errors.INVALID_REQUEST_PAYLOAD | Errors.USER_REFUSED_OP | Errors.API_VERSION_NOT_SUPPORTED | Errors.UNKNOWN_ERROR;
    };
    /**
     * Sign typed data using the wallet.
     * @param params The typed data to sign.
     * @returns An array of signatures as strings.
     */
    wallet_signTypedData: {
        params: TypedData & ApiVersionRequest;
        result: Signature;
        errors: Errors.INVALID_REQUEST_PAYLOAD | Errors.USER_REFUSED_OP | Errors.API_VERSION_NOT_SUPPORTED | Errors.UNKNOWN_ERROR;
    };
    /**
     * Get the list of supported RPC specification versions.
     * @returns An array of supported specification strings.
     */
    wallet_supportedSpecs: {
        params?: never;
        result: SpecVersion[];
    };
    /**
     * Returns a list of wallet api versions compatible with the wallet.
     * Notice this might be different from Starknet JSON-RPC spec
     * @returns An array of supported wallet api versions.
     */
    wallet_supportedWalletApi: {
        params?: never;
        result: API_VERSION[];
    };
}
export type RpcMessage = {
    [K in keyof RpcTypeToMessageMap]: {
        type: K;
    } & RpcTypeToMessageMap[K];
}[keyof RpcTypeToMessageMap];
export type IsParamsOptional<T extends keyof RpcTypeToMessageMap> = undefined extends RpcTypeToMessageMap[T]['params'] ? true : false;
export type RequestFnCall<T extends RpcMessage['type']> = {
    type: T;
} & (IsParamsOptional<T> extends true ? {
    params?: RpcTypeToMessageMap[T]['params'];
} : {
    params: RpcTypeToMessageMap[T]['params'];
});
export type RequestFn = <T extends RpcMessage['type']>(call: RequestFnCall<T>) => Promise<RpcTypeToMessageMap[T]['result']>;
//# sourceMappingURL=methods.d.ts.map