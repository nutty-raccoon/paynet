import type { NodeId } from "./node";

export type QuoteId = string;

export type PendingMintQuotesUpdateEvent = {
  node_id: NodeId,
  state: NodePendingMintQuotes,
}

export type NodePendingMintQuotes = {
  unpaid: QuoteId[],
  paid: QuoteId[],
}
