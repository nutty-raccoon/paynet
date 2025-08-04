import type { StarknetWindowObject, WalletProvider } from "@starknet-io/get-starknet-core";
export interface WalletProviderWithStoreVersion extends Omit<WalletProvider, "downloads"> {
    download: string;
}
export default function show({ discoveryWallets, installedWallets, lastWallet, authorizedWallets, enable, modalOptions, }: {
    lastWallet?: StarknetWindowObject;
    installedWallets?: StarknetWindowObject[];
    authorizedWallets?: StarknetWindowObject[];
    discoveryWallets?: WalletProviderWithStoreVersion[];
    enable?: (wallet: StarknetWindowObject | null) => Promise<StarknetWindowObject | null>;
    modalOptions?: {
        theme?: "light" | "dark" | "system";
    };
}): Promise<StarknetWindowObject | null>;
