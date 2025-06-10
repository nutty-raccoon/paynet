import { invoke } from "@tauri-apps/api/core";
import type { Balance, NodeBalances, NodeId } from "./types";

export async function getNodesBalance() {
     let n =  await invoke("get_nodes_balance")
       .then((message) => message as NodeBalances[] )
      .catch((error) => console.error(error));
  }

export async function addNode(nodeUrl: string) {
     const res = await invoke("add_node", {nodeUrl})
      .then((message) => message as [NodeId, Balance[]] )
      .catch((error) => {
        console.error(`failed to add node with url '${nodeUrl}':`, error);
      });

      return res;
}
