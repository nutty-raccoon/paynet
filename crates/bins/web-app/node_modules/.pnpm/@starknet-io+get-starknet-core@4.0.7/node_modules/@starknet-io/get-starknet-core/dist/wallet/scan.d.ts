import type { StarknetWindowObject } from "@starknet-io/types-js";
export declare function scanObjectForWallets(obj: Record<string, any>, isWalletObject: (wallet: any) => boolean): StarknetWindowObject[];
