import { invoke } from "@tauri-apps/api/core";
import type { Balance, NodeData, NodeId } from "./types";
import type { QuoteId } from "./types/quote";
import type { WadHistoryItem, Wads } from "./types/wad";

export async function getNodesBalance() {
     let res =  await invoke("get_nodes_balance")
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
      .then((message) => message as [NodeId, Balance[]] )
      .catch((error) => {
        console.error(`failed to add node with url '${nodeUrl}':`, error);
      });

      return res;
}

export type CreateMintQuoteResponse = {
  quoteId: QuoteId,
  paymentRequest: string,
}

export async function create_mint_quote(nodeId: NodeId, amount: string, asset: string) {
     const res = await invoke("create_mint_quote", {nodeId, amount, asset})
      .then((message) => message as CreateMintQuoteResponse )
      .catch((error) => {
        console.error(`failed to create mint quote:`, error);
      });

      return res
}

export async function redeem_quote(nodeId: NodeId, quoteId: QuoteId) {
      await invoke("redeem_quote", {nodeId, quoteId})
      .catch((error) => {
        console.error(`failed to redeem quote:`, error);
      });

      return ;
}

export async function create_wads(amount: string, asset: string) {
      const res = await invoke("create_wads", {amount, asset})
      .then((message) => message as Wads)
      .catch((error) => {
        console.error(`failed to create wads:`, error);
      });

      return res;
  
} 

export async function receive_wads(wads: string) {
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

export async function get_wad_history(limit?: number): Promise<WadHistoryItem[] | undefined> {
      const res = await invoke("get_wad_history", {limit})
      .then((message) => message as WadHistoryItem[])
      .catch((error) => {
        console.error("failed to get wad history:", error);
        return undefined;
      });

      return res;
} 

export async function sync_wads(): Promise<void> {
      await invoke("sync_wads")
      .catch((error) => {
        console.error("failed to sync wads:", error);
      });
} 


export async function refresh_node_keysets(nodeId: NodeId) {
      await invoke("refresh_node_keysets", {nodeId})
      .catch((error) => {
        console.error(`failed to refresh node keysets:`, error);
      });

      return;
}
