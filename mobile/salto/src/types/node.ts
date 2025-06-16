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

export type BalanceIncrease = {
  nodeId: NodeId,
  unit: string,
  amount: number
}
