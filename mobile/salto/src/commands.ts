import { invoke } from "@tauri-apps/api/core";
import type { Balance, NodeData, NodeId } from "./types";

export async function getNodesBalance() {
     let res =  await invoke("get_nodes_balance")
       .then((message) => message as NodeData[] )
      .catch((error) => console.error(error));

      console.log("getNodesData", res);

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

