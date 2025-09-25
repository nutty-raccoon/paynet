import type { NodeId, Unit } from "./node"

export type MintMethodSettings = {
  method: string,
  unit: Unit,
  minAmount?: number,
  maxAmount?: number,
  options: object,
}

export type MintSettings = {
  methods: [MintMethodSettings]
  disabled: boolean,
  
}
export type NodeMintMethodSettings = {
  nodeId: NodeId,
  settings?: MintSettings,
}
