<script lang="ts">
  import type { EventHandler } from "svelte/elements";
  import { formatBalance, unitPrecision } from "../../utils";
  import { createWads } from "../../commands";
  import type { Wads } from "../../types/wad";
  import { showSuccessToast } from "../../stores/toast";
  import { t } from "../../stores/i18n";

  interface Props {
    availableUnits: string[];
    availableBalances: Map<string, number>;
    onClose: () => void;
    onPaymentDataGenerated: (
      amount: string,
      asset: string,
      paymentData: Wads,
    ) => void;
  }

  let {
    availableUnits,
    availableBalances,
    onClose,
    onPaymentDataGenerated,
  }: Props = $props();

  let selectedUnit = $state<string>(
    availableUnits.length > 0 ? availableUnits[0] : "",
  );
  let paymentError = $state<string>("");
  let isSubmitting = $state<boolean>(false);

  let { asset, assetAmount } = $derived(
    formatBalance(selectedUnit, availableBalances.get(selectedUnit) || 0),
  );

  const handleFormSubmit: EventHandler<SubmitEvent, HTMLFormElement> = (
    event,
  ) => {
    event.preventDefault();
    
    // Prevent double submission
    if (isSubmitting) {
      return;
    }

    const form = event.target as HTMLFormElement;
    const formDataObject = new FormData(form);
    const inputAsset = formDataObject.get("payment-asset");
    const inputAmount = formDataObject.get("payment-amount");

    // Clear previous error
    paymentError = "";

    if (inputAmount && inputAsset) {
      const amountString = inputAmount.toString();

      const amountValue = parseFloat(amountString);
      if (amountValue <= 0) {
        paymentError = $t('modals.amountGreaterThanZero');
        return;
      }
      if (amountValue > assetAmount) {
        paymentError = `${$t('errors.amountCannotExceed')} ${assetAmount} ${selectedUnit}`;
        return;
      }

      isSubmitting = true;
      
      createWads(amountString, asset)
        .then((wads) => {
          if (!!wads) {
            showSuccessToast($t('send.paymentDataGenerated'));
            onPaymentDataGenerated(amountString, asset, wads);
          }
        })
        .catch((error) => {
          paymentError = $t('errors.failedGeneratePayment');
          console.error("Error creating wads:", error);
        })
        .finally(() => {
          isSubmitting = false;
        });
    }
  };
</script>

<div class="amount-form-container">
  <div class="method-indicator">
    <button class="back-button" onclick={onClose}>{$t('forms.backButton')}</button>
  </div>

  <form onsubmit={handleFormSubmit}>
    <div class="form-group">
      <label for="payment-asset">{$t('forms.currency')}</label>
      <select
        id="payment-asset"
        name="payment-asset"
        bind:value={selectedUnit}
        required
      >
        {#each availableUnits as unit}
          {@const formatted = formatBalance(unit, 0)}
          <option value={unit}>{formatted.asset}</option>
        {/each}
      </select>
      {#if selectedUnit}
        <span class="balance-info">
          {$t('modals.available')} {assetAmount}
          {asset}
        </span>
      {/if}
    </div>

    <div class="form-group">
      <label for="payment-amount">{$t('forms.amount')}</label>
      <input
        type="number"
        id="payment-amount"
        name="payment-amount"
        min="0"
        max={assetAmount}
        placeholder="0.0"
        step={1 / unitPrecision(selectedUnit)}
        required
      />
    </div>

    {#if paymentError}
      <div class="error-message">
        {paymentError}
      </div>
    {/if}

    <button type="submit" class="submit-button" disabled={isSubmitting}>
      {isSubmitting ? $t('send.generating') : $t('send.pickPaymentMethod')}
    </button>
  </form>
</div>

<style>
  .amount-form-container {
    position: relative;
  }

  .method-indicator {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #eee;
  }

  .back-button {
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 0.5rem;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .back-button:hover {
    background-color: #f0f0f0;
  }

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-group label {
    display: block;
    font-size: 0.9rem;
    margin-bottom: 0.5rem;
    color: #333;
    font-weight: 500;
  }

  .form-group select,
  .form-group input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    box-sizing: border-box;
    background-color: white;
  }

  .form-group select:focus,
  .form-group input:focus {
    border-color: #1e88e5;
    outline: none;
    box-shadow: 0 0 0 2px rgba(30, 136, 229, 0.2);
  }

  .balance-info {
    display: block;
    font-size: 0.8rem;
    color: #666;
    margin-top: 0.25rem;
    font-style: italic;
  }

  .submit-button {
    padding: 0.8rem 2rem;
    background-color: #1e88e5;
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    width: 100%;
    transition: background-color 0.2s;
    font-size: 1rem;
  }

  .submit-button:hover {
    background-color: #1976d2;
  }

  .submit-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }

  .submit-button:disabled:hover {
    background-color: #ccc;
  }

  .error-message {
    margin-bottom: 1rem;
    padding: 0.75rem;
    background-color: #ffebee;
    border: 1px solid #f44336;
    border-radius: 6px;
    color: #c62828;
    font-size: 0.9rem;
    font-weight: 500;
  }
</style>
