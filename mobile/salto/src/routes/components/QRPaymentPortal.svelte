<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import QRCode from "@castlenine/svelte-qrcode";
  import { UR, UREncoder } from "@gandlaf21/bc-ur";
  import { Buffer } from "buffer";
  import Portal from "./Portal.svelte";

  interface Props {
    paymentData: Buffer;
    onClose: () => void;
  }

  let { paymentData, onClose }: Props = $props();

  let partToDisplay = $state<string>();
  let windowWidth = $state(0);
  let windowHeight = $state(0);
  let qrSize = $state(280); // Default size

  // Update QR size reactively when window dimensions change
  $effect(() => {
    if (windowWidth === 0 || windowHeight === 0) {
      qrSize = 280;
      return;
    }

    // Calculate available space considering modal padding and other elements
    const availableWidth = Math.min(windowWidth - 32, 400 - 24); // Account for modal max-width and padding
    const availableHeight = windowHeight * 0.5; // Use 50% of viewport height for QR code

    // Use the smaller dimension to ensure QR code fits
    const maxSize = Math.min(availableWidth, availableHeight);

    // Set reasonable bounds
    qrSize = Math.max(200, Math.min(maxSize, 400));
  });

  // Update window dimensions
  const updateDimensions = () => {
    windowWidth = window.innerWidth;
    windowHeight = window.innerHeight;
  };

  // Initialize QR code sequence when payment data is available
  $effect(() => {
    if (!paymentData) return;

    const ur = UR.fromBuffer(paymentData);
    const encoder = new UREncoder(ur, 150, 0);
    let active = true;

    const updateQRCode = () => {
      if (!active) return;

      const part = encoder.nextPart().toString();
      partToDisplay = part;

      // Schedule next update
      setTimeout(updateQRCode, 150);
    };

    // Start the QR code updates
    updateQRCode();

    // Cleanup function - automatically called when effect is destroyed
    return () => {
      active = false;
      partToDisplay = undefined;
    };
  });

  onMount(() => {
    // Set initial dimensions
    updateDimensions();

    // Add resize listener
    window.addEventListener("resize", updateDimensions);
  });

  onDestroy(() => {
    // Remove resize listener
    window.removeEventListener("resize", updateDimensions);
  });

  const handleClose = () => {
    onClose();
  };
</script>

<Portal isOpen={true} onClose={handleClose} title="Payment QR Code">
  <div class="qr-code-section">
    {#if partToDisplay}
      {#key partToDisplay}
        <QRCode data={partToDisplay} size={qrSize} />
      {/key}
    {:else}
      <div
        class="loading-placeholder"
        style="width: {qrSize}px; height: {qrSize}px;"
      >
        <p>Generating QR code...</p>
      </div>
    {/if}
    <p class="qr-instructions">Scan this QR code to complete the payment</p>
  </div>

  <div class="actions">
    <button class="done-button" onclick={handleClose}>Done</button>
  </div>
</Portal>

<style>
  .qr-code-section {
    text-align: center;
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0;
  }

  .loading-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .loading-placeholder p {
    color: #666;
    font-style: italic;
    font-size: 1rem;
  }

  .qr-instructions {
    color: #666;
    margin-bottom: 1.5rem;
    font-size: 1rem;
    line-height: 1.4;
  }

  .actions {
    padding-top: 1rem;
    border-top: 1px solid #eee;
    margin: 0;
  }

  .done-button {
    width: 100%;
    padding: 1rem 2rem;
    background-color: #1e88e5;
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 12px;
    cursor: pointer;
    transition: background-color 0.2s;
    font-size: 1rem;
  }

  .done-button:hover {
    background-color: #1976d2;
  }
</style>
