import { invoke } from "@tauri-apps/api/core";
import type { NodeData, NodeId } from "./types";
import type { QuoteId } from "./types/quote";
import type { WadHistoryItem, Wads } from "./types/wad";
import { showErrorToast } from "./stores/toast";

export async function getPendingQuotes() {
     let res =  await invoke("get_pending_quotes")
       .then((message) => message as NodeData[] )
       .catch((error) => {
         console.log("Failed to get pending quotes:", error);
         showErrorToast("Failed to load pending quotes. Please try again.", error);
         return undefined;
       });
      return res;
  }

  export async function getCurrencies() {
    let res =  await invoke("get_currencies")
       .then((message) => message as string[])
       .catch((error) => {
         console.log("Failed to get currencies:", error);
         showErrorToast("Failed to load available currencies.", error);
         return undefined;
       });
      return res;
  }

  export async function setPriceProviderCurrency(currency: string) {
    await invoke("set_price_provider_currency", { newCurrency: currency })
      .catch((error) => {
        console.log("Failed to set price provider currency:", error);
        showErrorToast("Failed to update currency setting.", error);
      });
  }

export async function getTokensPrices() {
  let res = await invoke("get_tokens_prices")
    .then((message) => message as {})
    .catch((error) => {
      console.log("Failed to get token prices:", error);
      showErrorToast("Failed to load current token prices.", error);
      return undefined;
    });
  return res;
}

export async function addNode(nodeUrl: string) {
     const res = await invoke("add_node", {nodeUrl})
      .catch((error) => {
        console.log(`Failed to add node with url '${nodeUrl}':`, error);
        showErrorToast("Failed to add node. Please check the URL and try again.", error);
        return undefined;
      });

      return res;
}

export async function createMintQuote(nodeId: NodeId, amount: string, asset: string) {
     const res = await invoke("create_mint_quote", {nodeId, amount, asset})
      .catch((error) => {
        console.log(`Failed to create mint quote:`, error);
        showErrorToast("Failed to create deposit quote. Please try again.", error);
        return undefined;
      });

      return res
}

export async function createMeltQuote(nodeId: NodeId, amount: string, asset: string, to:  string) {
     const res = await invoke("create_melt_quote", {nodeId, method: "starknet", amount, asset, to})
      .catch((error) => {
        console.log(`Failed to create melt quote:`, error);
        showErrorToast("Failed to create withdrawal quote. Please try again.", error);
        return undefined;
      });

      return res
}

export async function payMintQuote(nodeId: NodeId, quoteId: QuoteId) {
     const res = await invoke("pay_mint_quote", {nodeId, quoteId})
      .catch((error) => {
        console.log(`Failed to pay mint quote:`, error);
        showErrorToast("Failed to process deposit payment. Please try again.", error);
        return undefined;
      });

      return res
}

export async function payMeltQuote(nodeId: NodeId, quoteId: QuoteId) {
     const res = await invoke("pay_melt_quote", {nodeId, quoteId})
      .catch((error) => {
        console.log(`Failed to pay melt quote:`, error);
        showErrorToast("Failed to process withdrawal payment. Please try again.", error);
        return undefined;
      });

      return res
}

export async function redeemQuote(nodeId: NodeId, quoteId: QuoteId) {
      await invoke("redeem_quote", {nodeId, quoteId})
      .catch((error) => {
        console.log(`Failed to redeem quote:`, error);
        showErrorToast("Failed to redeem tokens. Please try again.", error);
      });

      return ;
}

export async function createWads(amount: string, asset: string) {
      const res = await invoke("create_wads", {amount, asset})
      .then((message) => message as Wads)
      .catch((error) => {
        console.log(`Failed to create wads:`, error);
        showErrorToast("Failed to prepare payment. Please check your balance and try again.", error);
        return undefined;
      });

      return res;
  
} 

export async function receiveWads(wads: string) {
      const res = await invoke("receive_wads", {wads})
      .catch((error) => {
        console.log("Failed to receive wads:", error);
        showErrorToast("Failed to receive payment. Please check the payment data and try again.", error);
        return undefined;
      });

      return res;
} 

export type InitWalletResponse = {
  seedPhrase: string;
}

export async function checkWalletExists() {
  const res = await invoke("check_wallet_exists")
    .then((message) => message as boolean)
    .catch((error) => {
      console.log("Failed to check wallet exists:", error);
      showErrorToast("Failed to check wallet status. Please restart the app.", error);
      return false;
    });

  return res;
}

export async function initWallet() {
  const res = await invoke("init_wallet")
    .then((message) => message as InitWalletResponse)
    .catch((error) => {
      console.log("Failed to init wallet:", error);
      showErrorToast("Failed to create wallet. Please try again.", error);
      return undefined;
    });

  return res;
}

export async function restoreWallet(seedPhrase: string) {
  const res = await invoke("restore_wallet", { seedPhrase })
    .catch((error) => {
      console.log("Failed to restore wallet:", error);
      showErrorToast("Failed to restore wallet. Please check your seed phrase and try again.", error);
      return undefined;
    });

  return res;
}

export async function getSeedPhrase() {
  const res = await invoke("get_seed_phrase")
    .then((message) => message as string)
    .catch((error) => {
      console.log("Failed to get seed phrase:", error);
      showErrorToast("Failed to retrieve seed phrase. Please try again.", error);
      return undefined;
    });

  return res;
}

export async function getWadHistory(limit?: number): Promise<WadHistoryItem[] | undefined> {
      const res = await invoke("get_wad_history", {limit})
      .then((message) => message as WadHistoryItem[])
      .catch((error) => {
        console.log("Failed to get wad history:", error);
        showErrorToast("Failed to load transaction history. Please try again.", error);
        return undefined;
      });

      return res;
} 

export async function syncWads(): Promise<void> {
      await invoke("sync_wads")
      .catch((error) => {
        console.log("Failed to sync wads:", error);
        showErrorToast("Failed to sync transaction data. Please try again.", error);
      });
} 


export async function refreshNodeKeysets(nodeId: NodeId) {
      await invoke("refresh_node_keysets", {nodeId})
      .catch((error) => {
        console.log(`Failed to refresh node keysets:`, error);
        showErrorToast("Failed to refresh node configuration. Please try again.", error);
      });

      return;
}

export async function forgetNode(nodeId: NodeId, force: boolean) {
      await invoke("forget_node", {nodeId, force})
      .catch((error) => {
        console.log(`Failed to forget node:`, error);
        showErrorToast("Failed to remove node. Please try again.", error);
      });

      return;
}

