<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import QRCode from "@castlenine/svelte-qrcode";
  import { UR, UREncoder } from "@gandlaf21/bc-ur";
  import { Buffer } from "buffer";

  interface Props {
    paymentData: Buffer;
    onClose: () => void;
  }

  let { paymentData, onClose }: Props = $props();

  let partToDisplay = $state<string>();
  let showPortal = $state(false);
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
    showPortal = true;
    // Prevent body scroll
    document.body.style.overflow = "hidden";

    // Set initial dimensions
    updateDimensions();

    // Add resize listener
    window.addEventListener("resize", updateDimensions);
  });

  onDestroy(() => {
    // Restore body scroll
    document.body.style.overflow = "";

    // Remove resize listener
    window.removeEventListener("resize", updateDimensions);
  });

  const handleClose = () => {
    onClose();
  };
</script>

{#if showPortal}
  <div class="qr-payment-overlay">
    <div class="qr-payment-content">
      <div class="qr-payment-header">
        <h2>Payment QR Code</h2>
        <button class="close-button" onclick={handleClose}>✕</button>
      </div>

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
    </div>
  </div>
{/if}

<style>
  .qr-payment-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    z-index: 9999;
  }

  .qr-payment-content {
    background: white;
    border-radius: 16px;
    width: 100%;
    max-width: 400px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
  }

  .qr-payment-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 1.5rem 0;
    border-bottom: 1px solid #eee;
    margin-bottom: 1.5rem;
  }

  .qr-payment-header h2 {
    margin: 0;
    font-size: 1.5rem;
    color: #333;
    font-weight: 600;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: #666;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    transition: background-color 0.2s;
    line-height: 1;
  }

  .close-button:hover {
    background-color: #f0f0f0;
  }

  .qr-code-section {
    text-align: center;
    flex: 1;
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
    padding: 1.5rem;
    border-top: 1px solid #eee;
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
