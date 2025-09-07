import type { NodeId, Unit } from "./node";

export type QuoteId = string;

export type PendingQuoteData  = {
    id: string,
    unit: Unit,
    amount: number,
}

export type PendingQuotesUpdateEvent = {
  type: "mint" | "melt",
  nodeId: NodeId,
  mint?: {
    unpaid: PendingQuoteData[],
    paid: PendingQuoteData[],
  },
  melt?: {
    unpaid: PendingQuoteData[],
    pending: PendingQuoteData[],
  }
}

export type QuoteIdentifier = {
  nodeId: NodeId,
  quoteId: QuoteId
}

export type QuoteEvent = {
  type: "created" | "paid" | "redeemed" | "removed",
  quoteType: "mint" | "melt",
  nodeId: NodeId,
} & (
  | { type: "created", quote: PendingQuoteData }
  | { type: "paid" | "redeemed" | "removed", quoteId: QuoteId }
)

