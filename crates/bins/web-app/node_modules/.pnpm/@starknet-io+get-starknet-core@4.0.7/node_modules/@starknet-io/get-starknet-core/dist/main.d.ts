import type { GetStarknetOptions, GetStarknetResult } from "./types";
export type { StarknetWindowObject, AddDeclareTransactionParameters, AddDeclareTransactionResult, AddInvokeTransactionParameters, AddInvokeTransactionResult, AddStarknetChainParameters, RequestAccountsParameters, SwitchStarknetChainParameters, AccountDeploymentData, WatchAssetParameters, TypedData, RequestFn, RpcMessage, IsParamsOptional, RpcTypeToMessageMap, RequestFnCall, AccountChangeEventHandler, NetworkChangeEventHandler, WalletEventHandlers, WalletEvents, } from "@starknet-io/types-js";
export { scanObjectForWallets } from "./wallet/scan";
export { isWalletObject } from "./wallet/isWalletObject";
export type { BrowserStoreVersion, DisconnectOptions, GetStarknetOptions, GetStarknetResult, GetWalletOptions, OperatingSystemStoreVersion, WalletProvider, } from "./types";
export { ssrSafeWindow } from "./utils";
export declare function getStarknet(options?: Partial<GetStarknetOptions>): GetStarknetResult;
declare const _default: GetStarknetResult;
export default _default;
