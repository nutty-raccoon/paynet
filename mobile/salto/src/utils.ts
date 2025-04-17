import type { Balance } from "./types";

/**
 * Format a number as USD currency
 * @param balance The number to format as currency
 * @returns Formatted currency string
 */
export function formatBalance(balance: Balance): string {
  return `${balance.unit}: ${balance.amount}`
}
