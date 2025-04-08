  import { invoke } from "@tauri-apps/api/core";

export async function get_nodes_balance() {
     let n =  await invoke("get_nodes_balance")
       .then((message) => console.log(message))
      .catch((error) => console.error(error));
      // .then((message: Node[]) => {
  //       nodes = message;
  //     })
  //     .catch((error) => console.error(error));
  // })
  }
