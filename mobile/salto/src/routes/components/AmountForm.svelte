<script lang="ts">
  import type { EventHandler } from "svelte/elements";
  import { formatBalance, unitPrecision } from "../../utils";
  import { create_wads } from "../../commands";
  import { Buffer } from "buffer";

  interface Props {
    availableUnits: string[];
    availableBalances: Map<string, number>;
    onClose: () => void;
    onPaymentDataGenerated: (
      amount: string,
      asset: string,
      paymentData: any,
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
  let amount = $state<number>(0);
  let paymentError = $state<string>("");

  let { asset, amount: assetAmount } = $derived(
    formatBalance({
      unit: selectedUnit,
      amount: availableBalances.get(selectedUnit) || 0,
    }),
  );

  const handleFormSubmit: EventHandler<SubmitEvent, HTMLFormElement> = (
    event,
  ) => {
    event.preventDefault();
    const form = event.target as HTMLFormElement;
    const formDataObject = new FormData(form);
    const token = formDataObject.get("payment-token");
    const amount = formDataObject.get("payment-amount");

    // Clear previous error
    paymentError = "";

    if (amount && token) {
      const amountString = amount.toString();
      const amountValue = parseFloat(amountString);

      if (amountValue <= 0) {
        paymentError = "Amount must be greater than 0";
        return;
      }
      if (amountValue > assetAmount) {
        paymentError = `Amount cannot exceed ${assetAmount} ${selectedUnit}`;
        return;
      }

      create_wads(amountString, asset).then((val) => {
        if (!!val) {
          const messageBuffer = Buffer.from(val);
          onPaymentDataGenerated(amountString, token.toString(), messageBuffer);
        }
      });
    }
  };

  const handleUnitChange = (event: Event) => {
    const target = event.target as HTMLSelectElement;
    selectedUnit = target.value;
    // Reset amount when unit changes
    amount = 0;
  };
</script>

<div class="amount-form-container">
  <div class="method-indicator">
    <button class="back-button" onclick={onClose}>← Back</button>
  </div>

  <form onsubmit={handleFormSubmit}>
    <div class="form-group">
      <label for="payment-token">Currency</label>
      <select
        id="payment-token"
        name="payment-token"
        bind:value={selectedUnit}
        onchange={handleUnitChange}
        required
      >
        {#each availableUnits as unit}
          {@const formatted = formatBalance({ unit, amount: 0 })}
          <option value={unit}>{formatted.asset}</option>
        {/each}
      </select>
      {#if selectedUnit}
        <span class="balance-info">
          Available: {assetAmount}
          {asset}
        </span>
      {/if}
    </div>

    <div class="form-group">
      <label for="payment-amount">Amount</label>
      <input
        type="number"
        id="payment-amount"
        name="payment-amount"
        bind:value={amount}
        placeholder="0.0"
        min="0"
        max={assetAmount}
        step={1 / unitPrecision(selectedUnit)}
        required
      />
    </div>

    {#if paymentError}
      <div class="error-message">
        {paymentError}
      </div>
    {/if}

    <button type="submit" class="submit-button"> Pick a payment method </button>
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
