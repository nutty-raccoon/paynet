import type { Balance, BalanceChange, NodeData } from "./types";

/**
 * Format a balance into separate amount and unit strings
 * @param balance The balance to format
 * @returns Object with formatted amount and unit strings
 */
export function formatBalance(balance: Balance): {amount: number, asset: string} {
  switch(balance.unit) {
    case "millistrk":
      return { asset: "STRK", amount: balance.amount / 1000};
    case "gwei":
      return { asset: "ETH", amount: balance.amount / 1000000000};
    default:
      return {asset: balance.unit.toLocaleUpperCase(), amount: balance.amount};
   }
}

export function unitPrecision(unit: string): number {
  switch(unit) {
  case "millistrk":
    return 1000;
  case "gwei":
    return 1000000000;
  default:
    console.log("unknown unit:", unit);
    return 1;
  } 
}


export function increaseNodeBalance(nodes: NodeData[], balanceChange: BalanceChange) {
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

export function decreaseNodeBalance(nodes: NodeData[], balanceChange: BalanceChange) {
      let nodeToUpdate = nodes.find((n) => {
        return n.id == balanceChange.nodeId;
      });

      if (nodeToUpdate !== undefined) {
        const balanceToUpdate = nodeToUpdate.balances.find((b) => {
          return b.unit == balanceChange.unit;
        });
        if (!!balanceToUpdate) {
          if (balanceChange.amount > balanceToUpdate.amount) {
            console.log("error: balance decreased by more that its amount");
            balanceToUpdate.amount = 0;
          } else {
            balanceToUpdate.amount = balanceToUpdate.amount - balanceChange.amount;
          }
        } else {
        console.log("error: cannot decrease a balance not registered in the state");
        }
       } else {
        console.log("error: deposited on a node not registered in the state");
      }
}

export function computeTotalBalancePerUnit(nodes: NodeData[]): Map<string, number> {
  const map: Map<string, number> = new Map();
  nodes.forEach((n) => n.balances.forEach((b) => {
    let currentAmount = map.get(b.unit);
    if (!!currentAmount) {
      map.set(b.unit, currentAmount + b.amount);
    } else {
      map.set(b.unit, b.amount);
    }
  }))


  return map;
}
