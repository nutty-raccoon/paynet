import type { Balance, BalanceIncrease, NodeData } from "./types";

/**
 * Format a number as USD currency
 * @param balance The number to format as currency
 * @returns Formatted currency string
 */
export function formatBalance(balance: Balance): string {
  return `${balance.unit}: ${balance.amount}`
}

export function increaseNodeBalance(nodes: NodeData[], balanceChange: BalanceIncrease) {
      let nodeToUpdate = nodes.find((n) => {
        return n.id == balanceChange.nodeId;
      });

      if (nodeToUpdate !== undefined) {
        const balanceToUpdate = nodeToUpdate.balances.find((b) => {
          return b.unit == balanceChange.unit;
        });
        if (!!balanceToUpdate) {
          balanceToUpdate.amount = balanceToUpdate.amount + balanceChange.amount;
        } else {
          const newBalance = {
            unit: balanceChange.unit,
            amount: balanceChange.amount, 
          };
          nodeToUpdate.balances.push(newBalance);
        }
       } else {
        console.log("error: deposited on a node not registered in the state");
      }
}
