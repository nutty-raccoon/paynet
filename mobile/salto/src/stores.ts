import { isAvailable } from '@tauri-apps/plugin-nfc';
import { readable, type Readable } from 'svelte/store';
  import { platform } from "@tauri-apps/plugin-os";

export const isNFCAvailable: Readable<boolean> = readable(false, (set) => {
  isAvailable()
    .then(r => (r as any).available as boolean )
    .then(data => set(data))
    .catch(error => {
      console.error('Failed to read nfc availability:', error);
      set(false);
    });
});

const currentPlatform = platform();

export const isMobile = readable(false, (set) => {
  set(currentPlatform == "ios" || currentPlatform == "android");
});
