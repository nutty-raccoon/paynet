/**
 * Format a number as USD currency
 * @param balance The number to format as currency
 * @returns Formatted currency string
 */
export function formatBalance(balance: number): string {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
  }).format(balance);
}