<script lang="ts">
  import PaymentMethodChoice from "../components/PaymentMethodChoice.svelte";
  import { isNFCAvailable, isMobile } from "../..//stores.js";
  import NfcModal from "../components/NfcModal.svelte";
  import ScanModal from "../scan/ScanModal.svelte";

  const Modal = {
    METHOD_CHOICE: 0,
    NFC: 1,
    QR_CODE: 2,
  } as const;
  type Modal = (typeof Modal)[keyof typeof Modal];

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let currentModal = $state<Modal>(Modal.METHOD_CHOICE);

  const handleModalClose = () => {
    onClose();
  };

  const handleNFCChoice = () => {
    if (isNFCAvailable) {
      currentModal = Modal.NFC;
    } else {
      alert("NFC not available on your device");
    }
  };

  const handleQRCodeChoice = () => {
    if (isMobile) {
      currentModal = Modal.QR_CODE;
    } else {
      alert("qrcode scan only available on mobile");
    }
  };
</script>

<div class="modal-overlay">
  <div class="modal-content">
    <div class="modal-header">
      <h3>Receive Payment</h3>
      <button class="close-button" onclick={handleModalClose}>✕</button>
    </div>

    {#if currentModal === Modal.METHOD_CHOICE}
      <PaymentMethodChoice
        paymentStrings={null}
        onNFCChoice={handleNFCChoice}
        onQRCodeChoice={handleQRCodeChoice}
      />
    {:else if currentModal == Modal.NFC}
      <NfcModal
        onClose={() => (currentModal = Modal.METHOD_CHOICE)}
        isReceiving={true}
      />
    {:else if currentModal == Modal.QR_CODE}
      <ScanModal
        onSuccess={handleModalClose}
        onCancell={() => (currentModal = Modal.METHOD_CHOICE)}
      />
    {/if}
  </div>
</div>
