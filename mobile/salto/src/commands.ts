import { invoke } from "@tauri-apps/api/core";
import type { Balance, BalanceChange, NodeData, NodeId } from "./types";
import type { QuoteId } from "./types/quote";

export async function getNodesBalance() {
     let res =  await invoke("get_nodes_balance")
       .then((message) => message as NodeData[] )
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
      .then((message) => message as string)
      .catch((error) => {
        console.error(`failed to create wads:`, error);
      });

      return res;
  
} 

