export type NodeId = number;
export type Unit = string;

export type NodeData = {
  id: NodeId;
  url: string;
  balances: Balance[];
};

export type Balance = {
  unit: Unit;
  amount: number;
}

export type BalanceChange = {
  nodeId: NodeId,
  unit: Unit,
  amount: number
}
