import { readable, writable, type Readable } from 'svelte/store';
import { platform } from "@tauri-apps/plugin-os";

const currentPlatform = platform();

export const isMobile = readable(false, (set) => {
  set(currentPlatform == "ios" || currentPlatform == "android");
});

export const selectedCurrencyStored = writable<string>('usd');
export const fiatCurrenciesStored = writable<string[]>([]);
