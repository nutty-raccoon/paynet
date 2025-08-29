import type {  BalanceChange, NodeData, Unit } from "./types";
import type { Price } from "./types/price";

/**
 * Format a balance into separate amount and unit strings
 * @param balance The balance to format
 * @returns Object with formatted amount and unit strings
 */
export function formatBalance(unit: Unit, amount: number): {assetAmount: number, asset: string} {
  switch(unit) {
    case "m-strk":
      return { asset: "strk", assetAmount: amount / 1_000};
    case "gwei":
      return { asset: "eth", assetAmount: amount / 1_000_000_000};
    case "u-usdc":
      return { asset: "usdc", assetAmount: amount / 1_000_000};
    case "u-usdt":
      return { asset: "usdt", assetAmount: amount / 1_000_000};
    case "sat":
      return { asset: "wbtc", assetAmount: amount / 100_000_000};
    default:
      return {asset: unit.toLowerCase(), assetAmount: amount};
   }
}

export function unitPrecision(unit: string): number {
  switch(unit) {
  case "m-strk":
    return 1000;
  case "gwei":
    return 1_000_000_000;
  case "u-usdc":
    return 1_000_000;
  case "u-usdt":
    return 1_000_000;
  case "sat":
    return 100_000_000;
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

export function computeTotalBalancePerUnit(nodes: NodeData[]): Map<Unit, number> {
  const map: Map<Unit, number> = new Map();
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

export function getTotalAmountInDisplayCurrency(balances: Map<Unit, number>, prices: Price[])  {
    let totalAmount: number = 0;
    balances
      .entries()
      .forEach(([unit, amount]) => {
        const {asset, assetAmount: asset_amount} = formatBalance( unit, amount );

        let price = prices.find(
          (p) => asset === p.symbol,
        );
        if (!!price) {
          totalAmount += asset_amount * (price.value ? price.value : 0);
        }
      });

      return totalAmount;
  }
