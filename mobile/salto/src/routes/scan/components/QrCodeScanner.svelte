<script lang="ts">
  import { onMount } from "svelte";
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

  let scanner: Html5QrcodeScanner;
  onMount(() => {
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
  });

  $effect(() => {
    togglePause(paused);
  });

  function togglePause(paused: boolean): void {
    if (paused && scanner?.getState() === Html5QrcodeScannerState.SCANNING) {
      scanner?.pause();
    } else if (scanner?.getState() === Html5QrcodeScannerState.PAUSED) {
      scanner?.resume();
    }
  }
</script>

<div id="qr-scanner" class="w-full max-w-sm"></div>
