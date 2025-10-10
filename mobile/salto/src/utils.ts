import type {  Amount, BalanceChange, NodeData, Unit } from "./types";
import type { Price } from "./types/price";
import type { WadStatus } from "./types/wad";
import { get } from 'svelte/store';
import { t } from './stores/i18n';

/**
 * Format a balance into separate amount and unit strings
 * @param balance The balance to format
 * @returns Object with formatted amount and unit strings
 */
export function formatBalance(unit: Unit, amount: Amount): {assetAmount: number, asset: string} {
  return { asset: unitToAsset(unit), assetAmount: Number(amount) / unitPrecision(unit) }
}

export function unitPrecision(unit: Unit): number {
  switch(unit) {
  case "m-strk":
    return 1000;
  case "gwei":
    return 1_000_000_000;
  case "c-usdc":
    return 100;
  case "c-usdt":
    return 100;
  case "sat":
    return 1;
  default:
    console.log("unknown unit:", unit);
    return 1;
  } 
}

export function assetPrecision(unit: Unit): bigint {
  switch(unit) {
  case "m-strk":
    return 1_000_000_000_000_000_000n;
  case "gwei":
    return 1_000_000_000_000_000_000n;
  case "c-usdc":
    return 1_000_000n;
  case "c-usdt":
    return 1_000_000n;
  case "sat":
    return 1n;
  default:
    console.log("unknown unit:", unit);
    return 1n;
  } 
}

export function unitToAsset(unit: Unit): string {
  switch (unit) {
    case "m-strk":
      return "strk";
    case "gwei":
      return "eth";
    case "c-usdc":
      return "usdc";
    case "c-usdt":
      return "usdt";
    case "sat":
      return "sat";
    default:
      return unit.toLowerCase();
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
            balanceToUpdate.amount = 0n;
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

export function computeTotalBalancePerUnit(nodes: NodeData[]): Map<Unit, bigint> {
  const map: Map<Unit, bigint> = new Map();
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

export function getTotalAmountInDisplayCurrency(balances: Map<Unit, bigint>, prices: Price[])  {
    let totalAmount = 0;
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

export function isValidStarknetAddress(address: string): boolean {
  // Basic Starknet address validation
  // Addresses should start with 0x and be between 3 and 66 characters
  const addressRegex = /^0x[a-fA-F0-9]{1,63}$/;
  return addressRegex.test(address) && address.length >= 3 && address.length <= 66;
}

/**
 * Maps backend status values to internationalized display text
 * @param status - The backend status value (PENDING, FINISHED, FAILED)
 * @returns The translated status text
 */
export function getStatusDisplayText(status: WadStatus | string): string {
  const translate = get(t);
  
  switch (status.toUpperCase()) {
    case 'PENDING':
      return translate('history.pending');
    case 'FINISHED':
      return translate('history.completed');
    case 'FAILED':
      return translate('history.failed');
    default:
      return status; // Fallback to original status if not recognized
  }
}

export function divideBigIntToFloat(dividend: bigint, divisor: bigint, precision = 18) {
  if (divisor === 0n) {
    throw new Error("Division by zero");
  }
  
  // Handle sign
  const isNegative = (dividend < 0n) !== (divisor < 0n);
  dividend = dividend < 0n ? -dividend : dividend;
  divisor = divisor < 0n ? -divisor : divisor;
  
  // Integer part
  const integerPart = dividend / divisor;
  
  // Remainder for decimal part
  let remainder = dividend % divisor;
  
  // Calculate decimal part
  let decimalPart = '';
  for (let i = 0; i < precision && remainder !== 0n; i++) {
    remainder *= 10n;
    const digit = remainder / divisor;
    decimalPart += digit.toString();
    remainder = remainder % divisor;
  }
  
  // Construct result
  let result = integerPart.toString();
  if (decimalPart.length > 0) {
    result += '.' + decimalPart;
  }
  
  return isNegative ? '-' + result : result;
}
