<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    Html5QrcodeScanner,
    type Html5QrcodeResult,
    Html5QrcodeScanType,
    Html5QrcodeSupportedFormats,
    Html5QrcodeScannerState,
  } from "html5-qrcode";

  interface Props {
    width: number;
    height: number;
    paused: boolean;
    onCodeDetected: (decodedText: string) => void;
  }
  let { width, height, paused, onCodeDetected }: Props = $props();

  function onScanSuccess(
    decodedText: string,
    decodedResult: Html5QrcodeResult,
  ): void {
    onCodeDetected(decodedText);
  }

  // usually better to ignore and keep scanning
  function onScanFailure(message: string) {}

  let scanner: Html5QrcodeScanner | null = null;

  function initializeScanner(): void {
    if (scanner) return;

    scanner = new Html5QrcodeScanner(
      "qr-scanner",
      {
        fps: 24,
        qrbox: { width, height },
        aspectRatio: 1,
        supportedScanTypes: [Html5QrcodeScanType.SCAN_TYPE_CAMERA],
        formatsToSupport: [Html5QrcodeSupportedFormats.QR_CODE],
      },
      false, // non-verbose
    );
    scanner.render(onScanSuccess, onScanFailure);
  }

  function cleanupScanner(): void {
    if (scanner) {
      try {
        scanner.clear();
      } catch (error) {
        console.warn("Error clearing scanner:", error);
      }
      scanner = null;
    }
  }

  onMount(() => {
    if (!paused) {
      initializeScanner();
    }
  });

  onDestroy(() => {
    cleanupScanner();
  });

  $effect(() => {
    if (paused) {
      if (scanner?.getState() === Html5QrcodeScannerState.SCANNING) {
        scanner?.pause();
      }
    } else {
      if (!scanner) {
        initializeScanner();
      } else if (scanner?.getState() === Html5QrcodeScannerState.PAUSED) {
        scanner?.resume();
      }
    }
  });
</script>

<div id="qr-scanner" class="w-full max-w-sm"></div>
