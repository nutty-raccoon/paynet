import { VirtualWallet } from "../../types";
import { RpcMessage, StarknetWindowObject, WalletEventHandlers } from "@starknet-io/types-js";
import { Mutex } from "async-mutex";
interface MetaMaskProvider {
    isMetaMask: boolean;
    request(options: {
        method: string;
    }): Promise<void>;
}
export type Eip6963SupportedWallet = {
    provider: MetaMaskProvider | null;
};
declare class MetaMaskVirtualWallet implements VirtualWallet, Eip6963SupportedWallet, StarknetWindowObject {
    #private;
    id: string;
    name: string;
    icon: string;
    windowKey: string;
    provider: MetaMaskProvider | null;
    swo: StarknetWindowObject | null;
    lock: Mutex;
    version: string;
    constructor();
    /**
     * Load and resolve the `StarknetWindowObject`.
     *
     * @param windowObject The window object.
     * @returns A promise to resolve a `StarknetWindowObject`.
     */
    loadWallet(windowObject: Record<string, unknown>): Promise<StarknetWindowObject>;
    /**
     * Verify if the hosting machine supports the Wallet or not without loading the wallet itself.
     *
     * @param windowObject The window object.
     * @returns A promise that resolves to a boolean value to indicate the support status.
     */
    hasSupport(windowObject: Record<string, unknown>): Promise<boolean>;
    /**
     * Proxy the RPC request to the `this.swo` object.
     * Load the `this.swo` if not loaded.
     *
     * @param call The RPC API arguments.
     * @returns A promise to resolve a response of the proxy RPC API.
     */
    request<Data extends RpcMessage>(call: Omit<Data, "result">): Promise<Data["result"]>;
    /**
     * Subscribe the `accountsChanged` or `networkChanged` event.
     * Proxy the subscription to the `this.swo` object.
     * Load the `this.swo` if not loaded.
     *
     * @param event - The event name.
     * @param handleEvent - The event handler function.
     */
    on<Event extends keyof WalletEventHandlers>(event: Event, handleEvent: WalletEventHandlers[Event]): void;
    /**
     * Un-subscribe the `accountsChanged` or `networkChanged` event for a given handler.
     * Proxy the un-subscribe request to the `this.swo` object.
     * Load the `this.swo` if not loaded.
     *
     * @param event - The event name.
     * @param handleEvent - The event handler function.
     */
    off<Event extends keyof WalletEventHandlers>(event: Event, handleEvent: WalletEventHandlers[Event]): void;
}
declare const metaMaskVirtualWallet: MetaMaskVirtualWallet;
export { metaMaskVirtualWallet };
