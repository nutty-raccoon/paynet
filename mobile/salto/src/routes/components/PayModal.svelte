<script lang="ts">
  import type { EventHandler } from "svelte/elements";
  import { formatBalance, unitPrecision } from "../../utils";
  import { create_wads } from "../../commands";
  import QRPaymentPortal from "./QRPaymentPortal.svelte";
  import { Buffer } from "buffer";

  interface Props {
    isOpen: boolean;
    availableBalances: Map<string, number>;
    onClose: () => void;
  }

  let { isOpen, availableBalances, onClose }: Props = $props();

  let selectedUnit = $state<string>("millistrk");
  let amount = $state<number>(0);
  let paymentError = $state<string>("");
  let paymentData = $state<any>(null);

  // Get available units (those with balance > 0)
  let availableUnits = $derived(
    Array.from(availableBalances.entries())
      .filter(([_, balance]) => balance > 0)
      .map(([unit, _]) => unit),
  );

  // Reset form when modal opens/closes
  $effect(() => {
    if (isOpen) {
      selectedUnit = availableUnits.length > 0 ? availableUnits[0] : "";
      amount = 0;
      paymentError = "";
      paymentData = null;
    }
  });

  let { asset, amount: assetAmount } = $derived(
    formatBalance({
      unit: selectedUnit,
      amount: availableBalances.get(selectedUnit) || 0,
    }),
  );

  const onQRCodeClose = () => {
    paymentData = null;
  };

  const handleModalClose = () => {
    if (!!paymentData) {
      onQRCodeClose();
    }
    onClose();
  };

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
          const messageBuffer = Buffer.from(JSON.stringify(val));
          paymentData = messageBuffer;
        }
      });
    }
  };

  $effect(() => {
    if (!isOpen) {
      paymentError = "";
      paymentData = null; // Ensure QR component is properly cleaned up
    }
  });

  const handleUnitChange = (event: Event) => {
    const target = event.target as HTMLSelectElement;
    selectedUnit = target.value;
    // Reset amount when unit changes
    amount = 0;
  };
</script>

{#if isOpen}
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
      {:else if paymentData}
        <QRPaymentPortal {paymentData} onClose={onQRCodeClose} />
      {:else}
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

          <button type="submit" class="submit-button">Pay</button>
        </form>
      {/if}
    </div>
  </div>
{/if}

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
