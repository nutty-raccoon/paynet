<script lang="ts">
  import ReceivingMethodChoice from "./ReceivingMethodChoice.svelte";
  import ScanModal from "../scan/ScanModal.svelte";
  import { receiveWads } from "../../commands";
  import { readText } from "@tauri-apps/plugin-clipboard-manager";
  import { showSuccessToast, showErrorToast } from "../../stores/toast";

  const Modal = {
    METHOD_CHOICE: 0,
    QR_CODE: 1,
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

  const handleQRCodeChoice = () => {
    currentModal = Modal.QR_CODE;
  };

  const handlePasteChoice = async () => {
    try {
      const wads = await readText();
      if (!wads || wads.trim() === "") {
        showErrorToast("Clipboard is empty or contains no payment data");
        return;
      }

      const result = await receiveWads(wads);
      if (result !== undefined) {
        showSuccessToast("Payment received successfully");
        onClose();
      }
    } catch (error) {
      showErrorToast("Failed receive wad", error);
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
      <ReceivingMethodChoice
        onQRCodeChoice={handleQRCodeChoice}
        onPasteChoice={handlePasteChoice}
      />
    {:else if currentModal == Modal.QR_CODE}
      <ScanModal
        onSuccess={handleModalClose}
        onCancell={() => (currentModal = Modal.METHOD_CHOICE)}
      />
    {/if}
  </div>
</div>
