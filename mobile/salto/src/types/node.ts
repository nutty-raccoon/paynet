export type NodeId = number;

export type NodeBalances = {
  id: NodeId;
  url: string;
  balances: Balance[];
};

export type Balance = {
  unit: string;
  amount: number;
}
