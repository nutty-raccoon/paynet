import type { NodeId, Unit } from "./node";

export type QuoteId = string;

export type PendingMintQuoteData  = {
    id: string,
    unit: Unit,
    amount: number,
}

export type PendingMintQuotesUpdateEvent = {
  nodeId: NodeId,
  unpaid: PendingMintQuoteData[],
  paid: PendingMintQuoteData[],
}

export type MintQuoteIdentifier = {
  nodeId: NodeId,
  quoteId: QuoteId
}

