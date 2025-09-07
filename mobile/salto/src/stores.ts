import { derived, readable, writable } from 'svelte/store';
import { platform } from "@tauri-apps/plugin-os";
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Price } from './types/price';
import type { PendingQuoteData, PendingQuotesUpdateEvent, QuoteEvent} from './types/quote';
import type { NodeId } from './types';

const currentPlatform = platform();

export const isMobile = readable(false, (set) => {
  set(currentPlatform == "ios" || currentPlatform == "android");
});

export const displayCurrency = writable<string>('usd');

// Global token prices store with managed event listener
export const tokenPrices = readable<Price[] | null>(null, (set) => {
  let unlisten_new_prices: UnlistenFn | null = null;
  let unlisten_out_of_sync_price: UnlistenFn | null = null;
  
  // Set up the event listener
  const setupListener = async () => {
    try {
      unlisten_new_prices = await listen<Price[]>('new-price', (event) => {
        const prices = event.payload;
        set(prices);
      });
      unlisten_out_of_sync_price= await listen<null>("out-of-sync-price", (_event) => {
        set(null);
      })
    } catch (error) {
      console.error('Failed to set up token prices event listener:', error);
    }
  };
  
  // Initialize the listener
  setupListener();
  
  // Return cleanup function (though we won't actually call it in normal usage)
  return () => {
    if (unlisten_new_prices) {
      unlisten_new_prices();
    }
    if (unlisten_out_of_sync_price) {
      unlisten_out_of_sync_price();
    }
  };
});

export type NodePendingQuotes = {
  mint: {
    unpaid: PendingQuoteData[],
    paid: PendingQuoteData[],
  },
  melt: {
    unpaid: PendingQuoteData[],
    pending: PendingQuoteData[],
  }
}

export type MintQuoteCreatedEvent = {
  nodeId: NodeId,
  mintQuote: PendingQuoteData,
}

export const pendingQuotes = readable<Map<NodeId, NodePendingQuotes>>(
  new Map<NodeId, NodePendingQuotes>(),
   (_set, update
   ) => {
  let unlisten_updates: UnlistenFn | null = null;
  let unlisten_quote: UnlistenFn | null = null;

   const setupListener = async () => {
    try {
      unlisten_updates = await listen<PendingQuotesUpdateEvent>(
        "pending-quotes-updated",
        (event) => {
          const { type, nodeId, mint, melt } = event.payload;
          
          update((currentMap) => {
            // Create a new Map with all existing entries plus the updated one
            const newMap = new Map(currentMap);
            const existingQuotes = newMap.get(nodeId) || {
              mint: { unpaid: [], paid: [] },
              melt: { unpaid: [], pending: [] }
            };
            
            if (type === "mint" && mint) {
              existingQuotes.mint = mint;
            } else if (type === "melt" && melt) {
              existingQuotes.melt = melt;
            }
            
            newMap.set(nodeId, existingQuotes);
            return newMap;
          });
        }
      );

      unlisten_quote = await listen<QuoteEvent>(
        "quote",
        (event) => {
          const { type, quoteType, nodeId } = event.payload;

          if (quoteType === "mint") {
            switch (type) {
              case "created":
                if ("quote" in event.payload) {
                  const { quote } = event.payload;
                  update((currentMap) => {
                    const newMap = new Map(currentMap);
                    const nodeQuotes = newMap.get(nodeId) || {
                      mint: { unpaid: [], paid: [] },
                      melt: { unpaid: [], pending: [] }
                    };
                    nodeQuotes.mint.unpaid.push(quote);
                    newMap.set(nodeId, nodeQuotes);
                    return newMap;
                  });
                }
                break;
                
              case "paid":
                if ("quoteId" in event.payload) {
                  const { quoteId } = event.payload;
                  console.log("paid", event.payload);
                  
                  update((currentMap) => {
                    const newMap = new Map(currentMap);
                    const nodeQuotes = newMap.get(nodeId) || {
                      mint: { unpaid: [], paid: [] },
                      melt: { unpaid: [], pending: [] }
                    };
                    const quoteIndex = nodeQuotes.mint.unpaid.findIndex(quote => quote.id === quoteId);
                    if (quoteIndex !== -1) {
                      const [movedQuote] = nodeQuotes.mint.unpaid.splice(quoteIndex, 1);
                      nodeQuotes.mint.paid.push(movedQuote);
                      newMap.set(nodeId, nodeQuotes);
                    }
                    return newMap;
                  });
                }
                break;
                
              case "redeemed":
                if ("quoteId" in event.payload) {
                  const { quoteId } = event.payload;
                  console.log("redeem", event.payload);
                  
                  update((currentMap) => {
                    const newMap = new Map(currentMap);
                    const nodeQuotes = newMap.get(nodeId) || {
                      mint: { unpaid: [], paid: [] },
                      melt: { unpaid: [], pending: [] }
                    };
                    const quoteIndex = nodeQuotes.mint.paid.findIndex(quote => quote.id === quoteId);
                    if (quoteIndex !== -1) {
                      nodeQuotes.mint.paid.splice(quoteIndex, 1);
                      newMap.set(nodeId, nodeQuotes);
                    }
                    return newMap;
                  });
                }
                break;
                
              case "removed":
                if ("quoteId" in event.payload) {
                  const { quoteId } = event.payload;
                  
                  update((currentMap) => {
                    const newMap = new Map(currentMap);
                    const nodeQuotes = newMap.get(nodeId) || {
                      mint: { unpaid: [], paid: [] },
                      melt: { unpaid: [], pending: [] }
                    };
                    
                    // Remove from mint unpaid if present
                    const mintUnpaidIndex = nodeQuotes.mint.unpaid.findIndex(quote => quote.id === quoteId);
                    if (mintUnpaidIndex !== -1) {
                      nodeQuotes.mint.unpaid.splice(mintUnpaidIndex, 1);
                    }
                    
                    // Remove from mint paid if present
                    const mintPaidIndex = nodeQuotes.mint.paid.findIndex(quote => quote.id === quoteId);
                    if (mintPaidIndex !== -1) {
                      nodeQuotes.mint.paid.splice(mintPaidIndex, 1);
                    }
                    
                    newMap.set(nodeId, nodeQuotes);
                    return newMap;
                  });
                }
                break;
            }
          } else if (quoteType === "melt") {
            switch (type) {
              case "created":
                if ("quote" in event.payload) {
                  const { quote } = event.payload;
                  update((currentMap) => {
                    const newMap = new Map(currentMap);
                    const nodeQuotes = newMap.get(nodeId) || {
                      mint: { unpaid: [], paid: [] },
                      melt: { unpaid: [], pending: [] }
                    };
                    nodeQuotes.melt.unpaid.push(quote);
                    newMap.set(nodeId, nodeQuotes);
                    return newMap;
                  });
                }
                break;
                
              case "paid":
                if ("quoteId" in event.payload) {
                  const { quoteId } = event.payload;
                  update((currentMap) => {
                    const newMap = new Map(currentMap);
                    const nodeQuotes = newMap.get(nodeId) || {
                      mint: { unpaid: [], paid: [] },
                      melt: { unpaid: [], pending: [] }
                    };
                    const quoteIndex = nodeQuotes.melt.unpaid.findIndex(quote => quote.id === quoteId);
                    if (quoteIndex !== -1) {
                      const [movedQuote] = nodeQuotes.melt.unpaid.splice(quoteIndex, 1);
                      nodeQuotes.melt.pending.push(movedQuote);
                      newMap.set(nodeId, nodeQuotes);
                    }
                    return newMap;
                  });
                }
                break;
                
              case "redeemed":
              case "removed":
                if ("quoteId" in event.payload) {
                  const { quoteId } = event.payload;
                  update((currentMap) => {
                    const newMap = new Map(currentMap);
                    const nodeQuotes = newMap.get(nodeId) || {
                      mint: { unpaid: [], paid: [] },
                      melt: { unpaid: [], pending: [] }
                    };
                    
                    // Remove from unpaid
                    const unpaidIndex = nodeQuotes.melt.unpaid.findIndex(quote => quote.id === quoteId);
                    if (unpaidIndex !== -1) {
                      nodeQuotes.melt.unpaid.splice(unpaidIndex, 1);
                    }
                    
                    // Remove from pending
                    const pendingIndex = nodeQuotes.melt.pending.findIndex(quote => quote.id === quoteId);
                    if (pendingIndex !== -1) {
                      nodeQuotes.melt.pending.splice(pendingIndex, 1);
                    }
                    
                    newMap.set(nodeId, nodeQuotes);
                    return newMap;
                  });
                }
                break;
            }
          }
        }
      );
    } catch (error) {
      console.error('Failed to set up pending mint quotes event listener:', error);
    }
  };
;

  // Initialize the listener
  setupListener();

  // Return cleanup function
  return () => {
    if (unlisten_updates) {
      unlisten_updates();
    }
    if (unlisten_quote) {
      unlisten_quote();
    }
  };
});

// Derived store to check if nodes have pending quotes
export const nodesWithPendingQuotes = derived(pendingQuotes, ($pendingQuotes) => {
  const nodesWithQuotes = new Set<NodeId>();
  
  $pendingQuotes.forEach((quotes, nodeId) => {
    if (quotes.mint.unpaid.length > 0 || quotes.mint.paid.length > 0 ||
        quotes.melt.unpaid.length > 0 || quotes.melt.pending.length > 0) {
      nodesWithQuotes.add(nodeId);
    }
  });
  
  return nodesWithQuotes;
});
