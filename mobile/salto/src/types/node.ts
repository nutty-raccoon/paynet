export type NodeId = number;
export type Unit = string;
export type Amount = bigint;

export type NodeIdAndUrl = {
  id: NodeId;
  url: string;
};

export type NodeData = {
  id: NodeId;
  url: string;
  balances: Balance[];

};

export type Balance = {
  unit: Unit;
  amount: Amount;
}

export type BalanceChange = {
  nodeId: NodeId;
  unit: Unit;
  amount: Amount;
}
