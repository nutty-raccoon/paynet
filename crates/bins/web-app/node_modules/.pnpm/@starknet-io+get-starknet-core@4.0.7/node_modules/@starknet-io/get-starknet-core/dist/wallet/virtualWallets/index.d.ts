import type { VirtualWallet } from "../../types";
import type { StarknetWindowObject } from "@starknet-io/types-js";
declare const virtualWallets: VirtualWallet[];
declare function initiateVirtualWallets(windowObject: Record<string, unknown>): void;
declare function resolveVirtualWallet(windowObject: Record<string, unknown>, virtualWallet: VirtualWallet): Promise<StarknetWindowObject>;
export { initiateVirtualWallets, resolveVirtualWallet, virtualWallets };
