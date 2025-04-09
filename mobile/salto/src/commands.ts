  import { invoke } from "@tauri-apps/api/core";

export async function getNodesBalance() {
     let n =  await invoke("get_nodes_balance")
       .then((message) => console.log(message))
      .catch((error) => console.error(error));
  }

export async function addNode(nodeUrl: string) {
     const res = await invoke("add_node", {nodeUrl})
      .then((message) => message as number)
      .catch((error) => {
        console.error(`failed to add node with url '${nodeUrl}':`, error);
        return null;
      });

      return res;
}
