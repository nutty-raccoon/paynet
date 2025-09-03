import { derived, readable, writable, type Readable } from 'svelte/store';
import { platform } from "@tauri-apps/plugin-os";
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Price } from './types/price';
import type { MintQuoteIdentifier, PendingMintQuoteData, PendingMintQuotesUpdateEvent, QuoteId} from './types/quote';
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

export type NodePendingMintQuotes = {
  unpaid: PendingMintQuoteData[],
  paid: PendingMintQuoteData[],
}

export type MintQuoteCreatedEvent = {
  nodeId: NodeId,
  mintQuote: PendingMintQuoteData,
}

export const pendingMintQuotes = readable<Map<NodeId, NodePendingMintQuotes>>(
  new Map<NodeId, NodePendingMintQuotes>(),
   (_set, update
   ) => {
  let unlisten_updates: UnlistenFn | null = null;
  let unlisten_created: UnlistenFn | null = null;
  let unlisten_paid: UnlistenFn | null = null;
  let unlisten_redeemed: UnlistenFn | null = null;

   const setupListener = async () => {
    try {
      unlisten_updates = await listen<PendingMintQuotesUpdateEvent>(
        "pending-mint-quote-updated",
        (event) => {
          const { nodeId, unpaid, paid } = event.payload;
          
          update((currentMap) => {
            // Create a new Map with all existing entries plus the updated one
            const newMap = new Map(currentMap);
            newMap.set(nodeId, {unpaid, paid});
            return newMap;
          });
        }
      );

      unlisten_created = await listen<MintQuoteCreatedEvent>(
        "mint-quote-created",
        (event) => {
          const { nodeId, mintQuote} = event.payload;
          
          update((currentMap) => {
            // Create a new Map with all existing entries plus the updated one
            const newMap = new Map(currentMap);
            const nodeLists = newMap.get(nodeId) || { unpaid: [], paid: [] };
            nodeLists.unpaid.push(mintQuote)
            newMap.set(nodeId, nodeLists);
            return newMap;
          });
        }
      );

      unlisten_paid = await listen<MintQuoteIdentifier>(
        "mint-quote-paid",
        (event) => {
          const { nodeId, quoteId} = event.payload;

          console.log("paid", event.payload);
          
          update((currentMap) => {
            const newMap = new Map(currentMap);
            const nodeLists = newMap.get(nodeId) || { unpaid: [], paid: [] };
            const quoteIndex = nodeLists.unpaid.findIndex(quote => quote.id === quoteId);
            if (quoteIndex !== -1) {
              const [movedQuote] = nodeLists.unpaid.splice(quoteIndex, 1);      
              nodeLists.paid.push(movedQuote);
              newMap.set(nodeId, nodeLists);
            }
            return newMap;
          });
        }
      );

      unlisten_redeemed = await listen<MintQuoteIdentifier>(
        "mint-quote-redeemed",
        (event) => {
          const { nodeId, quoteId} = event.payload;
          
          console.log("redeem", event.payload);
          update((currentMap) => {
            const newMap = new Map(currentMap);
            const nodeLists = newMap.get(nodeId) || { unpaid: [], paid: [] };
            const quoteIndex = nodeLists.paid.findIndex(quote => quote.id === quoteId);
            if (quoteIndex !== -1) {
              nodeLists.paid.splice(quoteIndex, 1);      
              newMap.set(nodeId, nodeLists);
            }
            return newMap;
          });
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
    if (unlisten_created) {
      unlisten_created();
    }
    if (unlisten_paid) {
      unlisten_paid();
    }
    if (unlisten_redeemed) {
      unlisten_redeemed();
    }
  };
});

// Derived store to check if nodes have pending quotes
export const nodesWithPendingQuotes = derived(pendingMintQuotes, ($pendingMintQuotes) => {
  const nodesWithQuotes = new Set<NodeId>();
  
  $pendingMintQuotes.forEach((quotes, nodeId) => {
    if (quotes.unpaid.length > 0 || quotes.paid.length > 0) {
      nodesWithQuotes.add(nodeId);
    }
  });
  
  return nodesWithQuotes;
});
