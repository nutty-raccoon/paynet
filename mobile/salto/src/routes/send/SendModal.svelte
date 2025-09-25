<script lang="ts">
  import QRPaymentPortal from "./QRPaymentPortal.svelte";
  import AmountForm from "./AmountForm.svelte";
  import SendingMethodChoice from "./SendingMethodChoice.svelte";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { t } from "../../stores/i18n";
  import type { Wads } from "../../types/wad";

  const SelectedMethod = {
    NONE: 0,
    QR_CODE: 1,
  } as const;
  type SelectedMethod = (typeof SelectedMethod)[keyof typeof SelectedMethod];

  interface Props {
    availableBalances: Map<string, number>;
    onClose: () => void;
  }

  let { availableBalances, onClose }: Props = $props();

  let wads = $state<Wads | null>(null);
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

  const handleCopyChoice = async (wads: string) => {
    await writeText(wads);
  };

  const selectMethod = (modal: SelectedMethod) => {
    selectedMethod = modal;
  };

  const handlePaymentDataGenerated = (
    amountString: string,
    assetString: string,
    w: Wads,
  ) => {
    paymentStrings = [amountString, assetString];
    wads = w;
  };
</script>

<div class="modal-overlay">
  <div class="modal-content">
    <div class="modal-header">
      <h3>{$t('send.title')}</h3>
      <button class="close-button" onclick={handleModalClose}>✕</button>
    </div>

    {#if selectedMethod == SelectedMethod.NONE}
      {#if !wads}
        {#if availableUnits.length === 0}
          <div class="no-balance-message">
            <p>{$t('errors.insufficientBalance')}</p>
            <button class="close-button-alt" onclick={onClose}>{$t('common.cancel')}</button>
          </div>
        {:else}
          <AmountForm
            {availableUnits}
            {availableBalances}
            onClose={() => {}}
            onPaymentDataGenerated={handlePaymentDataGenerated}
          />
        {/if}
      {:else}
        <SendingMethodChoice
          {paymentStrings}
          onQRCodeChoice={() => selectMethod(SelectedMethod.QR_CODE)}
          onCopyChoice={() => handleCopyChoice(wads as Wads)}
        />
      {/if}
    {:else if selectedMethod === SelectedMethod.QR_CODE}
      {#if !!wads}
        <QRPaymentPortal
          data={wads}
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
