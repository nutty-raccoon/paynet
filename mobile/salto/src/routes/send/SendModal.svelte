<script lang="ts">
  import QRPaymentPortal from "./QRPaymentPortal.svelte";
  import NfcModal from "../components/NfcModal.svelte";
  import AmountForm from "./AmountForm.svelte";
  import SendingMethodChoice from "./SendingMethodChoice.svelte";
  import { isNFCAvailable } from "../../stores.js";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  const SelectedMethod = {
    NONE: 0,
    NFC: 1,
    QR_CODE: 2,
    COPY: 3,
  } as const;
  type SelectedMethod = (typeof SelectedMethod)[keyof typeof SelectedMethod];

  interface Props {
    availableBalances: Map<string, number>;
    onClose: () => void;
  }

  let { availableBalances, onClose }: Props = $props();

  let paymentData = $state<string | null>(null);
  let paymentStrings = $state<null | [string, string]>(null);

  // What to show
  let selectedMethod = $state<SelectedMethod>(SelectedMethod.NONE);

  // Get available units (those with balance > 0)
  let availableUnits = $derived(
    Array.from(availableBalances.entries())
      .filter(([_, balance]) => balance > 0)
      .map(([unit, _]) => unit),
  );

  const handleModalClose = () => {
    onClose();
  };

  const handleNFCChoice = async () => {
    if (isNFCAvailable) {
      selectedMethod = SelectedMethod.NFC;
    } else {
      alert("NFC not available on your device");
    }
  };

  const handleCopyChoice = async (wads: string) => {
    await writeText(wads);
  };

  const selectMethod = (modal: SelectedMethod) => {
    selectedMethod = modal;
  };

  const handlePaymentDataGenerated = (
    amountString: string,
    assetString: string,
    data: string,
  ) => {
    paymentStrings = [amountString, assetString];
    paymentData = data;
  };
</script>

<div class="modal-overlay">
  <div class="modal-content">
    <div class="modal-header">
      <h3>Make Payment</h3>
      <button class="close-button" onclick={handleModalClose}>✕</button>
    </div>

    {#if selectedMethod == SelectedMethod.NONE}
      {#if availableUnits.length === 0}
        <div class="no-balance-message">
          <p>No funds available for payment. Please deposit tokens first.</p>
          <button class="close-button-alt" onclick={onClose}>Close</button>
        </div>
      {:else if !paymentData}
        <AmountForm
          {availableUnits}
          {availableBalances}
          onClose={() => {}}
          onPaymentDataGenerated={handlePaymentDataGenerated}
        />
      {:else}
        <SendingMethodChoice
          {paymentStrings}
          onNFCChoice={handleNFCChoice}
          onQRCodeChoice={() => selectMethod(SelectedMethod.QR_CODE)}
          onCopyChoice={() => handleCopyChoice(paymentData as string)}
        />
      {/if}
    {:else if !!paymentData}
      {#if selectedMethod === SelectedMethod.NFC}
        <NfcModal
          isReceiving={false}
          onClose={() => selectMethod(SelectedMethod.NONE)}
        />
      {:else if selectedMethod === SelectedMethod.QR_CODE}
        <QRPaymentPortal
          {paymentData}
          onClose={() => selectMethod(SelectedMethod.NONE)}
        />
      {:else}
        Error
      {/if}
    {/if}
  </div>
</div>

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  .modal-content {
    background: white;
    border-radius: 12px;
    width: 90%;
    max-width: 400px;
    padding: 1.5rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.5rem;
    color: #333;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    color: #666;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .close-button:hover {
    background-color: #f0f0f0;
  }

  .no-balance-message {
    text-align: center;
    padding: 1rem 0;
  }

  .no-balance-message p {
    color: #666;
    margin-bottom: 1.5rem;
    font-size: 1rem;
  }

  .close-button-alt {
    padding: 0.8rem 2rem;
    background-color: #666;
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .close-button-alt:hover {
    background-color: #555;
  }
</style>
