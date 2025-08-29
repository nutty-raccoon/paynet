import { readable, writable, type Readable } from 'svelte/store';
import { platform } from "@tauri-apps/plugin-os";
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Price } from './types/price';

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

