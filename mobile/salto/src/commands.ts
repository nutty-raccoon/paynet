import { invoke } from "@tauri-apps/api/core";
import type { NodeData, NodeId } from "./types";
import type { QuoteId } from "./types/quote";
import type { WadHistoryItem, Wads } from "./types/wad";

export async function getNodesBalance() {
     let res =  await invoke("get_nodes_balance")
       .then((message) => message as NodeData[] )
       .catch((error) => console.error(error));
      return res;
  }

export async function getPendingQuotes() {
     let res =  await invoke("get_pending_quotes")
       .then((message) => message as NodeData[] )
       .catch((error) => console.error(error));
      return res;
  }

  export async function getCurrencies() {
    let res =  await invoke("get_currencies")
       .then((message) => message as string[])
       .catch((error) => console.error(error));
      return res;
  }

  export async function setPriceProviderCurrency(currency: string) {
    await invoke("set_price_provider_currency", { newCurrency: currency })
      .catch((error) => console.error(error));
  }

export async function getTokensPrices() {
  let res = await invoke("get_tokens_prices")
    .then((message) => message as {})
    .catch((error) => console.error(error));
  return res;
}

export async function addNode(nodeUrl: string) {
     const res = await invoke("add_node", {nodeUrl})
      .catch((error) => {
        console.error(`failed to add node with url '${nodeUrl}':`, error);
      });

      return res;
}

export async function createMintQuote(nodeId: NodeId, amount: string, asset: string) {
     const res = await invoke("create_mint_quote", {nodeId, amount, asset})
      .catch((error) => {
        console.error(`failed to create mint quote:`, error);
      });

      return res
}

export async function createMeltQuote(nodeId: NodeId, amount: string, asset: string, to:  string) {
     const res = await invoke("create_melt_quote", {nodeId, method: "starknet", amount, asset, to})
      .catch((error) => {
        console.error(`failed to create melt quote:`, error);
      });

      return res
}

export async function payMintQuote(nodeId: NodeId, quoteId: QuoteId) {
     const res = await invoke("pay_mint_quote", {nodeId, quoteId})
      .catch((error) => {
        console.error(`failed to pay mint quote:`, error);
      });

      return res
}

export async function payMeltQuote(nodeId: NodeId, quoteId: QuoteId) {
     const res = await invoke("pay_melt_quote", {nodeId, quoteId})
      .catch((error) => {
        console.error(`failed to pay melt quote:`, error);
      });

      return res
}

export async function redeemQuote(nodeId: NodeId, quoteId: QuoteId) {
      await invoke("redeem_quote", {nodeId, quoteId})
      .catch((error) => {
        console.error(`failed to redeem quote:`, error);
      });

      return ;
}

export async function createWads(amount: string, asset: string) {
      const res = await invoke("create_wads", {amount, asset})
      .then((message) => message as Wads)
      .catch((error) => {
        console.error(`failed to create wads:`, error);
      });

      return res;
  
} 

export async function receiveWads(wads: string) {
      const res = await invoke("receive_wads", {wads})
      .catch((error) => {
        console.error("failed to receive wads:", error);
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
      console.error("failed to check wallet exists:", error);
      return false;
    });

  return res;
}

export async function initWallet() {
  const res = await invoke("init_wallet")
    .then((message) => message as InitWalletResponse)
    .catch((error) => {
      console.error("failed to init wallet:", error);
    });

  return res;
}

export async function restoreWallet(seedPhrase: string) {
  const res = await invoke("restore_wallet", { seedPhrase })
    .catch((error) => {
      console.error("failed to restore wallet:", error);
    });

  return res;
}

export async function getSeedPhrase() {
  const res = await invoke("get_seed_phrase")
    .then((message) => message as string)
    .catch((error) => {
      console.error("failed to get seed phrase:", error);
    });

  return res;
}

export async function getWadHistory(limit?: number): Promise<WadHistoryItem[] | undefined> {
      const res = await invoke("get_wad_history", {limit})
      .then((message) => message as WadHistoryItem[])
      .catch((error) => {
        console.error("failed to get wad history:", error);
        return undefined;
      });

      return res;
} 

export async function syncWads(): Promise<void> {
      await invoke("sync_wads")
      .catch((error) => {
        console.error("failed to sync wads:", error);
      });
} 


export async function refreshNodeKeysets(nodeId: NodeId) {
      await invoke("refresh_node_keysets", {nodeId})
      .catch((error) => {
        console.error(`failed to refresh node keysets:`, error);
      });

      return;
}
