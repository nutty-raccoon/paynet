export type NodeId = number;

export type NodeData = {
  id: NodeId;
  url: string;
  balances: Balance[];
};

export type Balance = {
  unit: string;
  amount: number;
}
