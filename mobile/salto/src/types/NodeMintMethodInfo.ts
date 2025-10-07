import type { Amount, NodeId, Unit } from "./node"

export type MintUnitSettings = {
  minAmount: string,
  maxAmount: string,
}

export type NodeMintMethodSettings = {
  nodeId: NodeId,
  disabled: boolean,
  settings: Record<Unit, MintUnitSettings[]>,
}

export type MintSettings = {
  disabled: boolean,
  methods: Array<{
    unit: Unit,
    minAmount: Amount,
    maxAmount: Amount,
  }>,
}
