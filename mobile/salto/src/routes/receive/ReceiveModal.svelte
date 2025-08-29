<script lang="ts">
  import ReceivingMethodChoice from "./ReceivingMethodChoice.svelte";
  import { isMobile } from "../..//stores.js";
  import ScanModal from "../scan/ScanModal.svelte";
  import { receiveWads } from "../../commands";
  import { readText } from "@tauri-apps/plugin-clipboard-manager";

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
    if (isMobile) {
      currentModal = Modal.QR_CODE;
    } else {
      alert("qrcode scan only available on mobile");
    }
  };

  const handlePasteChoice = async () => {
    const wads = await readText();
    await receiveWads(wads);
    onClose();
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
