export type NodeBalances = {
  id: number;
  url: string;
  balances: Balance[];
};

export type Balance = {
  unit: string;
  amount: number;
}
