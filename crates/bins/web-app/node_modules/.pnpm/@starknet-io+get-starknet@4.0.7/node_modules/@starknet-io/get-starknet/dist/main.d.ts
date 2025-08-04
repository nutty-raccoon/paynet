import { type BrowserStoreVersion, type DisconnectOptions, type GetWalletOptions, type OperatingSystemStoreVersion, type StarknetWindowObject } from "@starknet-io/get-starknet-core";
export type { StarknetWindowObject, DisconnectOptions, } from "@starknet-io/get-starknet-core";
export interface ConnectOptions extends GetWalletOptions {
    modalMode?: "alwaysAsk" | "canAsk" | "neverAsk";
    modalTheme?: "light" | "dark" | "system";
    storeVersion?: BrowserStoreVersion;
    osVersion?: OperatingSystemStoreVersion;
}
export declare const connect: ({ modalMode, storeVersion, osVersion, modalTheme, ...restOptions }?: ConnectOptions) => Promise<StarknetWindowObject | null>;
export declare function disconnect(options?: DisconnectOptions): Promise<void>;
