import type { ChainId } from '../api/index.js';
export type AccountChangeEventHandler = (accounts?: string[]) => void;
export type NetworkChangeEventHandler = (chainId?: ChainId, accounts?: string[]) => void;
export interface WalletEventHandlers {
    accountsChanged: AccountChangeEventHandler;
    networkChanged: NetworkChangeEventHandler;
}
export type WalletEvents = {
    [E in keyof WalletEventHandlers]: {
        type: E;
        handler: WalletEventHandlers[E];
    };
}[keyof WalletEventHandlers];
export type WalletEventListener = <E extends keyof WalletEventHandlers>(event: E, handleEvent: WalletEventHandlers[E]) => void;
//# sourceMappingURL=events.d.ts.map