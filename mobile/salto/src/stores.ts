import { readable, type Readable } from 'svelte/store';
import { platform } from "@tauri-apps/plugin-os";

const currentPlatform = platform();

export const isMobile = readable(false, (set) => {
  set(currentPlatform == "ios" || currentPlatform == "android");
});
