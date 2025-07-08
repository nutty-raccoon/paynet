<script lang="ts">
  import QRPaymentPortal from "./QRPaymentPortal.svelte";
  import NfcModal from "./NfcModal.svelte";
  import AmountForm from "./AmountForm.svelte";
  import PaymentMethodChoice from "./PaymentMethodChoice.svelte";
  import { isNFCAvailable } from "../..//stores.js";

  const Modal = {
    AMOUNT_FORM: 0,
    METHOD_CHOICE: 1,
    NFC: 2,
    QR_CODE: 3,
  } as const;
  type Modal = (typeof Modal)[keyof typeof Modal];

  interface Props {
    availableBalances: Map<string, number>;
    onClose: () => void;
  }

  let { availableBalances, onClose }: Props = $props();

  let paymentData = $state<any>(null);
  let paymentStrings = $state<null | [string, string]>(null);

  // What to show
  let currentModal = $state<Modal>(Modal.AMOUNT_FORM);

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
      currentModal = Modal.NFC;
    } else {
      alert("NFC not available on your device");
    }
  };

  const openModal = (modal: Modal) => {
    currentModal = modal;
  };

  const handlePaymentDataGenerated = (
    amountString: string,
    assetString: string,
    data: any,
  ) => {
    paymentStrings = [amountString, assetString];
    paymentData = data;
    // Next step of the process
    currentModal = Modal.METHOD_CHOICE;
  };
</script>

<div class="modal-overlay">
  <div class="modal-content">
    <div class="modal-header">
      <h3>Make Payment</h3>
      <button class="close-button" onclick={handleModalClose}>✕</button>
    </div>

    {#if availableUnits.length === 0}
      <div class="no-balance-message">
        <p>No funds available for payment. Please deposit tokens first.</p>
        <button class="close-button-alt" onclick={onClose}>Close</button>
      </div>
    {:else if currentModal === Modal.AMOUNT_FORM}
      <AmountForm
        {availableUnits}
        {availableBalances}
        onClose={() => openModal(Modal.METHOD_CHOICE)}
        onPaymentDataGenerated={handlePaymentDataGenerated}
      />
    {:else if currentModal == Modal.METHOD_CHOICE}
      <PaymentMethodChoice
        {paymentStrings}
        onNFCChoice={handleNFCChoice}
        onQRCodeChoice={() => openModal(Modal.QR_CODE)}
      />
    {:else if currentModal === Modal.NFC}
      <NfcModal
        isReceiving={false}
        onClose={() => openModal(Modal.METHOD_CHOICE)}
      />
    {:else if currentModal === Modal.QR_CODE && paymentData}
      <QRPaymentPortal
        {paymentData}
        onClose={() => openModal(Modal.METHOD_CHOICE)}
      />
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
