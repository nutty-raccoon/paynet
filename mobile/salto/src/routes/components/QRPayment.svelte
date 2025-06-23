<script lang="ts">
  import QRCode from "@castlenine/svelte-qrcode";
  import { UR, UREncoder } from "@gandlaf21/bc-ur";
  import { Buffer } from "buffer";

  interface Props {
    paymentData: Buffer;
    onClose: () => void;
  }

  let { paymentData, onClose }: Props = $props();

  let partToDisplay = $state<string>();

  // Initialize QR code sequence when payment data is available
  $effect(() => {
    if (!paymentData) return;

    const ur = UR.fromBuffer(paymentData);
    const encoder = new UREncoder(ur, 100, 0);
    let active = true;

    const updateQRCode = () => {
      if (!active) return;

      console.log("update qrcode");
      const part = encoder.nextPart().toString();
      partToDisplay = part;

      // Schedule next update
      setTimeout(updateQRCode, 1000);
    };

    // Start the QR code updates
    updateQRCode();

    // Cleanup function - automatically called when effect is destroyed
    return () => {
      active = false;
      partToDisplay = undefined;
    };
  });
</script>

<div class="qr-code-container">
  <h4>Payment QR Code</h4>
  <div class="qr-code-wrapper">
    {#if partToDisplay}
      {#key partToDisplay}
        <QRCode data={partToDisplay} size={200} />
      {/key}
    {:else}
      <div class="loading-placeholder">
        <p>Generating QR code...</p>
      </div>
    {/if}
  </div>
  <p class="qr-instructions">Scan this QR code to complete the payment</p>
  <button class="close-button" onclick={onClose}>Done</button>
</div>

<style>
  .qr-code-container {
    text-align: center;
    padding: 1rem 0;
  }

  .qr-code-container h4 {
    margin: 0 0 1.5rem 0;
    font-size: 1.25rem;
    color: #333;
    font-weight: 600;
  }

  .qr-code-wrapper {
    display: flex;
    justify-content: center;
    align-items: center;
    margin-bottom: 1.5rem;
    padding: 1rem;
    background-color: #f9f9f9;
    border-radius: 8px;
    border: 1px solid #eee;
    min-height: 220px;
  }

  .loading-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 200px;
    height: 200px;
  }

  .loading-placeholder p {
    color: #666;
    font-style: italic;
  }

  .qr-instructions {
    color: #666;
    margin-bottom: 1.5rem;
    font-size: 0.9rem;
    line-height: 1.4;
  }

  .close-button {
    padding: 0.8rem 2rem;
    background-color: #666;
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .close-button:hover {
    background-color: #555;
  }
</style>
