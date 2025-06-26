<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { goto } from "$app/navigation";
  import {
    checkPermissions,
    requestPermissions,
    cancel,
  } from "@tauri-apps/plugin-barcode-scanner";
  import { URDecoder } from "@gandlaf21/bc-ur";
  import QrCodeScanner from "./components/QrCodeScanner.svelte";
  import Portal from "../components/Portal.svelte";

  let scanningInProgress = $state(false);
  let percentageEstimate = $state("");
  let originalHtmlStyle = "";
  let decoder = $state(new URDecoder());
  let paused = $state(true);
  let isPortalOpen = $state(false);

  function onCodeDetected(decodedText: string) {
    decoder.receivePart(decodedText);
    const estimatedPercentComplete = decoder.estimatedPercentComplete();
    percentageEstimate = (estimatedPercentComplete * 100).toFixed(0) + "%";

    if (decoder.isComplete()) {
      paused = true;
      if (decoder.isSuccess()) {
        // Get the UR representation of the message
        const ur = decoder.resultUR();
        // Decode the CBOR message to a Buffer
        const decoded = ur.decodeCBOR();
        // get the original message, assuming it was a JSON object
        const originalMessage = JSON.parse(decoded.toString());
      } else {
        // log and handle the error
        const error = decoder.resultError();
        console.log("Error found while decoding", error);
      }
    }
  }

  async function scanQRCode() {
    try {
      const permission = await checkPermissions();
      if (permission == "granted") {
        paused = false;
      } else {
        const permission = await requestPermissions();
        if (permission == "granted") {
          paused = false;
        } else {
          return "Permission denied";
        }
      }
    } catch (error) {
      console.error("QR code scanning failed:", error);
      return JSON.stringify(error);
    }
  }

  async function cancelScanning() {
    if (scanningInProgress) {
      try {
        await cancel();
        scanningInProgress = false;
        return "Scanning cancelled";
      } catch (error) {
        console.error("Failed to cancel scanning:", error);
        scanningInProgress = false;
        return "Cancel failed";
      }
    }
    return "No scanning in progress";
  }

  async function handleCancel() {
    await cancelScanning();
    goto("/");
  }

  const handlePortalClose = () => {
    goto("/");
  };

  onMount(() => {
    isPortalOpen = true;
    // Start scanning immediately when page loads
    scanQRCode();
  });

  onDestroy(() => {
    // Restore original styles if component is destroyed during scanning
    if (typeof document !== "undefined") {
      const html = document.documentElement;

      html.style.backgroundColor = originalHtmlStyle;
    }
  });
</script>

<Portal
  isOpen={isPortalOpen}
  onClose={handlePortalClose}
  backgroundColor="rgba(0, 0, 0, 0.95)"
>
  <div class="scan-content">
    <p class="scan-instructions">Point your camera at a QR code</p>

    <QrCodeScanner {onCodeDetected} {paused} />

    {#if percentageEstimate}
      <div class="scan-result">
        <h3>Scanned:</h3>
        <p>{percentageEstimate}</p>
      </div>
    {/if}

    <button class="cancel-button" onclick={handleCancel}>Cancel</button>
  </div>
</Portal>

<style>
  .scan-content {
    color: white;
    text-align: center;
  }

  .scan-instructions {
    margin: 0 0 2rem 0;
    font-size: 1rem;
    opacity: 0.8;
  }

  .scan-result {
    background-color: rgba(255, 255, 255, 0.1);
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 2rem;
  }

  .scan-result h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.2rem;
    color: #4caf50;
  }

  .scan-result p {
    margin: 0;
    word-break: break-all;
    font-family: monospace;
    font-size: 0.9rem;
  }

  .cancel-button {
    background-color: #d32f2f;
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
    padding: 0.8rem 2rem;
    border: none;
    border-radius: 50px;
    cursor: pointer;
    transition:
      background-color 0.2s,
      transform 0.1s;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
    width: 100%;
  }

  .cancel-button:hover {
    background-color: #b71c1c;
  }

  .cancel-button:active {
    transform: scale(0.98);
    background-color: #8e0000;
  }
</style>
