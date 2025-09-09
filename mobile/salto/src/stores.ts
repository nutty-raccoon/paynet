import { derived, readable, writable } from 'svelte/store';
import { platform } from "@tauri-apps/plugin-os";
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { Price } from './types/price';
import type { PendingQuoteData  } from './types/quote';
import type { NodeId, NodeData } from './types';

export const currentPlatform = platform();

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
      console.log('📡 Setting up token prices event listeners...');
      
      unlisten_new_prices = await listen<Price[]>('new-price', (event) => {
        console.log('💰 Received new-price event with', event.payload?.length || 0, 'prices');
        const prices = event.payload;
        set(prices);
      });
      
      unlisten_out_of_sync_price = await listen<null>("out-of-sync-price", (_event) => {
        console.log('⚠️ Received out-of-sync-price event');
        set(null);
      });
      
      console.log('✅ Token prices event listeners set up successfully');
    } catch (error) {
      console.error('❌ Failed to set up token prices event listener:', error);
    }
  };
  
  // Initialize the store and listener
  setupListener();

  // Return cleanup function
  return () => {
    console.log('🧹 Cleaning up token prices store');
    if (unlisten_new_prices) {
      unlisten_new_prices();
    }
    if (unlisten_out_of_sync_price) {
      unlisten_out_of_sync_price()
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

type MintPendingQuotes = {
  nodeId: NodeId,
  unpaid: PendingQuoteData[],
  paid: PendingQuoteData[],
}

type MeltPendingQuotes = {
  nodeId: NodeId,
  unpaid: PendingQuoteData[],
  pending: PendingQuoteData[],
}

type MintOrMeltQuote =
  | { mint: MintPendingQuotes }
  | { melt: MeltPendingQuotes }

class QuotesPoller {
  private intervalId: number | null = null;
  private isPolling = false;
  private readonly pollInterval = 10000; // 10 seconds
  private isCurrentlyPolling = false; // Track if a poll operation is in progress

  constructor(private updateCallback: (quotes: Map<NodeId, NodePendingQuotes>) => void) {
    console.log('🔄 QuotesPoller created');
  }

  start(): void {
    if (this.isPolling) {
      console.log('⚠️ QuotesPoller.start() called but already polling');
      return;
    }
    
    console.log('▶️ Starting QuotesPoller');
    this.isPolling = true;
    
    // Poll immediately on start
    this.pollQuotes();
    
    // Set up regular polling
    this.intervalId = setInterval(() => {
      console.log('⏰ Regular poll interval triggered');
      this.pollQuotes();
    }, this.pollInterval);
    
    console.log('✅ QuotesPoller started with', this.pollInterval + 'ms interval');
  }

  stop(): void {
    if (!this.isPolling) {
      console.log('⚠️ QuotesPoller.stop() called but not currently polling');
      return;
    }
    
    console.log('⏹️ Stopping QuotesPoller');
    this.isPolling = false;
    
    if (this.intervalId !== null) {
      clearInterval(this.intervalId);
      this.intervalId = null;
      console.log('🗑️ Cleared QuotesPoller interval');
    }
  }

  triggerImmediatePoll(): void {
    console.log('🚀 triggerImmediatePoll() called, isPolling:', this.isPolling, 'isCurrentlyPolling:', this.isCurrentlyPolling);
    
    if (this.isPolling) {
      if (this.isCurrentlyPolling) {
        console.log('⚠️ Skipping immediate poll - another poll operation is already in progress');
        return;
      }
      console.log('⚡ Executing immediate poll');
      this.pollQuotes();
    } else {
      console.log('⚠️ Skipping immediate poll - poller is not active');
    }
  }

  private async pollQuotes(): Promise<void> {
    if (!this.isPolling) {
      console.log('⚠️ pollQuotes() called but poller is stopped');
      return;
    }
    
    if (this.isCurrentlyPolling) {
      console.log('⚠️ pollQuotes() called but another poll is already in progress');
      return;
    }
    
    console.log('🔍 Starting quotes poll...');
    const startTime = performance.now();
    this.isCurrentlyPolling = true;
    
    try {
      console.log('📞 Invoking get_pending_quotes...');
      const pendingQuotesData = await invoke<MintOrMeltQuote[]>('get_pending_quotes');
      console.log('📨 Received pending quotes data:', pendingQuotesData?.length || 0, 'items');
      
      const quotesMap = this.transformQuotesToMap(pendingQuotesData);
      console.log('🗺️ Transformed to map with', quotesMap.size, 'nodes');
      
      console.log('📤 Calling update callback...');
      this.updateCallback(quotesMap);
      
      const duration = performance.now() - startTime;
      console.log('✅ Quotes poll completed successfully in', duration.toFixed(2) + 'ms');
      
    } catch (error) {
      const duration = performance.now() - startTime;
      console.error('❌ Failed to poll pending quotes after', duration.toFixed(2) + 'ms:', error);
      // Continue polling despite errors - the error might be temporary
    } finally {
      this.isCurrentlyPolling = false;
      console.log('🔓 Poll operation completed, isCurrentlyPolling set to false');
    }
  }

  private transformQuotesToMap(pendingQuotesData: MintOrMeltQuote[]): Map<NodeId, NodePendingQuotes> {
    console.log('🔄 Transforming', pendingQuotesData?.length || 0, 'quotes to map...');
    const quotesMap = new Map<NodeId, NodePendingQuotes>();
    
    if (!pendingQuotesData) {
      console.log('⚠️ No pending quotes data provided');
      return quotesMap;
    }
    
    for (const quote of pendingQuotesData) {
      if ('mint' in quote) {
        const mintQuote = quote.mint;
        console.log('🪙 Processing mint quote for node:', mintQuote.nodeId);
        const existing = quotesMap.get(mintQuote.nodeId) || {
          mint: { unpaid: [], paid: [] },
          melt: { unpaid: [], pending: [] }
        };
        existing.mint.unpaid = mintQuote.unpaid;
        existing.mint.paid = mintQuote.paid;
        quotesMap.set(mintQuote.nodeId, existing);
      } else if ('melt' in quote) {
        const meltQuote = quote.melt;
        console.log('🔥 Processing melt quote for node:', meltQuote.nodeId);
        const existing = quotesMap.get(meltQuote.nodeId) || {
          mint: { unpaid: [], paid: [] },
          melt: { unpaid: [], pending: [] }
        };
        existing.melt.unpaid = meltQuote.unpaid;
        existing.melt.pending = meltQuote.pending;
        quotesMap.set(meltQuote.nodeId, existing);
      } else {
        console.warn('⚠️ Unknown quote type:', quote);
      }
    }
    
    console.log('✅ Transform completed, returning map with', quotesMap.size, 'entries');
    return quotesMap;
  }
}

class NodeBalancesPoller {
  private intervalId: number | null = null;
  private isPolling = false;
  private readonly pollInterval = 10000; // 10 seconds
  private isCurrentlyPolling = false; // Track if a poll operation is in progress

  constructor(private updateCallback: (nodes: NodeData[]) => void) {
    console.log('💰 NodeBalancesPoller created');
  }

  start(): void {
    if (this.isPolling) {
      console.log('⚠️ NodeBalancesPoller.start() called but already polling');
      return;
    }
    
    console.log('▶️ Starting NodeBalancesPoller');
    this.isPolling = true;
    
    // Poll immediately on start
    this.pollBalances();
    
    // Set up regular polling
    this.intervalId = setInterval(() => {
      console.log('⏰ Regular balance poll interval triggered');
      this.pollBalances();
    }, this.pollInterval);
    
    console.log('✅ NodeBalancesPoller started with', this.pollInterval + 'ms interval');
  }

  stop(): void {
    if (!this.isPolling) {
      console.log('⚠️ NodeBalancesPoller.stop() called but not currently polling');
      return;
    }
    
    console.log('⏹️ Stopping NodeBalancesPoller');
    this.isPolling = false;
    
    if (this.intervalId !== null) {
      clearInterval(this.intervalId);
      this.intervalId = null;
      console.log('🗑️ Cleared NodeBalancesPoller interval');
    }
  }

  triggerImmediatePoll(): void {
    console.log('🚀 Balance triggerImmediatePoll() called, isPolling:', this.isPolling, 'isCurrentlyPolling:', this.isCurrentlyPolling);
    
    if (this.isPolling) {
      if (this.isCurrentlyPolling) {
        console.log('⚠️ Skipping immediate balance poll - another poll operation is already in progress');
        return;
      }
      console.log('⚡ Executing immediate balance poll');
      this.pollBalances();
    } else {
      console.log('⚠️ Skipping immediate balance poll - poller is not active');
    }
  }

  private async pollBalances(): Promise<void> {
    if (!this.isPolling) {
      console.log('⚠️ pollBalances() called but poller is stopped');
      return;
    }
    
    if (this.isCurrentlyPolling) {
      console.log('⚠️ pollBalances() called but another poll is already in progress');
      return;
    }
    
    console.log('💰 Starting balances poll...');
    const startTime = performance.now();
    this.isCurrentlyPolling = true;
    
    try {
      console.log('📞 Invoking get_nodes_balance...');
      const nodesBalanceData = await invoke<NodeData[]>('get_nodes_balance');
      console.log('📨 Received node balance data:', nodesBalanceData?.length || 0, 'nodes');
      
      console.log('📤 Calling balance update callback...', nodesBalanceData);
      this.updateCallback(nodesBalanceData || []);
      
      const duration = performance.now() - startTime;
      console.log('✅ Balance poll completed successfully in', duration.toFixed(2) + 'ms');
      
    } catch (error) {
      const duration = performance.now() - startTime;
      console.error('❌ Failed to poll node balances after', duration.toFixed(2) + 'ms:', error);
      // Continue polling despite errors - the error might be temporary
    } finally {
      this.isCurrentlyPolling = false;
      console.log('🔓 Balance poll operation completed, isCurrentlyPolling set to false');
    }
  }
}

export const pendingQuotes = readable<Map<NodeId, NodePendingQuotes>>(
  new Map<NodeId, NodePendingQuotes>(),
  (set) => {
    console.log('🏗️ Setting up pendingQuotes store...');
    let poller: QuotesPoller | null = null;
    let unlisten_trigger: UnlistenFn | null = null;

    const setup = async () => {
      console.log('⚙️ Setting up pendingQuotes poller and listeners...');
      
      // Create poller with update callback
      poller = new QuotesPoller((quotes) => {
        console.log('📊 Updating pendingQuotes store with', quotes.size, 'nodes');
        set(quotes);
      });

      // Start polling when first subscriber connects
      poller.start();

      // Set up trigger event listener
      try {
        console.log('👂 Setting up trigger-pending-quote-poll listener...');
        unlisten_trigger = await listen('trigger-pending-quote-poll', (event) => {
          console.log('🔔 Received trigger-pending-quote-poll event:', event);
          console.log('⚡ Triggering immediate quote poll...');
          poller?.triggerImmediatePoll();
        });
        console.log('✅ trigger-pending-quote-poll listener set up successfully');
      } catch (error) {
        console.error('❌ Failed to set up pending quote poll trigger listener:', error);
      }
    };

    setup();

    // Return cleanup function - stops polling when last subscriber disconnects
    return () => {
      console.log('🧹 Cleaning up pendingQuotes store');
      if (poller) {
        poller.stop();
      }
      if (unlisten_trigger) {
        unlisten_trigger();
      }
    };
  }
);

// Derived store to check if nodes have pending quotes
export const nodesWithPendingQuotes = derived(pendingQuotes, ($pendingQuotes) => {
  console.log('🔍 Deriving nodesWithPendingQuotes from', $pendingQuotes.size, 'nodes');
  const nodesWithQuotes = new Set<NodeId>();
  
  $pendingQuotes.forEach((quotes, nodeId) => {
    const hasQuotes = quotes.mint.unpaid.length > 0 || quotes.mint.paid.length > 0 ||
        quotes.melt.unpaid.length > 0 || quotes.melt.pending.length > 0;
    
    if (hasQuotes) {
      console.log('📋 Node', nodeId, 'has pending quotes');
      nodesWithQuotes.add(nodeId);
    }
  });
  
  console.log('📊 Found', nodesWithQuotes.size, 'nodes with pending quotes');
  return nodesWithQuotes;
});

// Global node balances store with managed polling
// In stores.ts, replace the nodeBalances store with:
export const nodeBalances = readable<NodeData[]>([], (set) => {
  console.log('🏗️ Setting up nodeBalances store...');
  let poller: NodeBalancesPoller | null = null;
  let unlisten_trigger: UnlistenFn | null = null;

  const setup = async () => {
    console.log('⚙️ Setting up nodeBalances poller and listeners...');
    
    // Create poller with update callback
    poller = new NodeBalancesPoller((nodes) => {
      console.log('💰 Updating nodeBalances store with', nodes.length, 'nodes');
      set(nodes);
    });

    // Start polling immediately - always keep it active
    poller.start();

    // Set up trigger event listener
    try {
      console.log('👂 Setting up trigger-balance-poll listener...');
      unlisten_trigger = await listen('trigger-balance-poll', (event) => {
        console.log('🔔 Received trigger-balance-poll event:', event);
        console.log('⚡ Triggering immediate balance poll...');
        poller?.triggerImmediatePoll();
      });
      console.log('✅ trigger-balance-poll listener set up successfully');
    } catch (error) {
      console.error('❌ Failed to set up balance poll trigger listener:', error);
    }
  };

  setup();

  // Return cleanup function - stops polling when last subscriber disconnects
  return () => {
    console.log('🧹 Cleaning up nodeBalances store - keeping poller active');
    if (poller) {
      poller.stop();
    }
    if (unlisten_trigger) {
      unlisten_trigger();
    }
  };
});

// Derived store for total balance across all nodes
export const totalBalance = derived(nodeBalances, ($nodeBalances) => {
  console.log('💯 Calculating total balance from', $nodeBalances.length, 'nodes');
  const totalBalanceMap = new Map<string, number>();
  
  $nodeBalances.forEach((node) => {
    node.balances.forEach((balance) => {
      const currentAmount = totalBalanceMap.get(balance.unit) || 0;
      totalBalanceMap.set(balance.unit, currentAmount + balance.amount);
    });
  });
  
  console.log('💯 Total balance calculated for', totalBalanceMap.size, 'units');
  return totalBalanceMap;
});
