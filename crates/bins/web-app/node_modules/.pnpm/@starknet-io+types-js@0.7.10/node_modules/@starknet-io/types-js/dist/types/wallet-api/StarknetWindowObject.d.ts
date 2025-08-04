import type { WalletEventListener } from './events.js';
import type { RequestFn } from './methods.js';
export interface StarknetWindowObject {
    id: string;
    name: string;
    version: string;
    icon: string | {
        dark: string;
        light: string;
    };
    request: RequestFn;
    on: WalletEventListener;
    off: WalletEventListener;
}
//# sourceMappingURL=StarknetWindowObject.d.ts.map