<script lang="ts">
  import type { EventHandler } from "svelte/elements";
  import type { NodeData } from "../../types";
  import { createMeltQuote } from "../../commands";
  import { formatBalance, isValidStarknetAddress } from "../../utils";

  interface Props {
    selectedNode: NodeData;
    onClose: () => void;
  }

  let { selectedNode, onClose }: Props = $props();
  let withdrawError = $state<string>("");
  let selectedAsset = $state<string>("");

  // Get available balances with formatted data
  let availableBalances = $derived(
    selectedNode.balances
      .map((balance) => ({
        ...balance,
        formatted: formatBalance(balance.unit, balance.amount),
      }))
      .filter((balance) => balance.formatted.assetAmount > 0), // Only show assets with positive balance
  );

  // Get maximum withdrawable amount for selected asset
  let maxWithdrawable = $derived.by(() => {
    const balance = availableBalances.find(
      (b) => b.formatted.asset === selectedAsset,
    );
    return balance ? balance.formatted.assetAmount : 0;
  });

  const handleFormSubmit: EventHandler<SubmitEvent, HTMLFormElement> = (
    event,
  ) => {
    event.preventDefault();
    const form = event.target as HTMLFormElement;
    const formData = new FormData(form);

    const amount = parseFloat(formData.get("withdraw-amount") as string);
    const asset = formData.get("withdraw-asset") as string;
    const toAddress = formData.get("withdraw-to") as string;

    // Clear previous errors
    withdrawError = "";

    // Validation checks
    if (!asset) {
      withdrawError = "Please select an asset";
      return;
    }

    if (amount <= 0) {
      withdrawError = "Amount must be greater than 0";
      return;
    }

    if (amount > maxWithdrawable) {
      withdrawError = `Insufficient balance. Maximum withdrawable: ${maxWithdrawable} ${asset.toUpperCase()}`;
      return;
    }

    if (!toAddress.trim()) {
      withdrawError = "Please enter a destination address";
      return;
    }

    if (!isValidStarknetAddress(toAddress.trim())) {
      withdrawError = "Please enter a valid Starknet address (0x...)";
      return;
    }

    // Create melt quote
    createMeltQuote(
      selectedNode.id,
      amount.toString(),
      asset,
      toAddress.trim(),
    );
    onClose();
  };

  function setMaxAmount() {
    const input = document.getElementById(
      "withdraw-amount",
    ) as HTMLInputElement;
    if (input && maxWithdrawable > 0) {
      input.value = maxWithdrawable.toString();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  // Reset error when modal closes
  $effect(() => {
    if (!selectedNode) {
      withdrawError = "";
      selectedAsset = "";
    }
  });
</script>

<div class="modal-backdrop" onclick={handleBackdropClick}>
  <div class="modal-content">
    <div class="modal-header">
      <h3>Withdraw Tokens</h3>
      <button class="close-button" onclick={onClose}>✕</button>
    </div>

    <form onsubmit={handleFormSubmit}>
      <div class="form-group">
        <label for="withdraw-asset">Asset</label>
        <select
          id="withdraw-asset"
          name="withdraw-asset"
          bind:value={selectedAsset}
          required
        >
          <option value="">Select asset...</option>
          {#each availableBalances as balance}
            <option value={balance.formatted.asset}>
              {balance.formatted.asset.toUpperCase()}
              (Available: {balance.formatted.assetAmount})
            </option>
          {/each}
        </select>
      </div>

      <div class="form-group">
        <label for="withdraw-amount">Amount</label>
        <div class="amount-input-group">
          <input
            type="number"
            id="withdraw-amount"
            name="withdraw-amount"
            placeholder="0.0"
            min="0"
            max={maxWithdrawable || undefined}
            step="any"
            required
          />
          {#if selectedAsset && maxWithdrawable > 0}
            <button type="button" class="max-button" onclick={setMaxAmount}>
              MAX
            </button>
          {/if}
        </div>
        {#if selectedAsset && maxWithdrawable > 0}
          <div class="balance-info">
            <p>Available: {maxWithdrawable} {selectedAsset.toUpperCase()}</p>
          </div>
        {/if}
      </div>

      <div class="form-group">
        <label for="withdraw-to">To Address</label>
        <input
          type="text"
          id="withdraw-to"
          name="withdraw-to"
          placeholder="0x..."
          required
        />
      </div>

      <div class="withdraw-info">
        <p>Withdrawing from: {selectedNode.url}</p>
      </div>

      {#if withdrawError}
        <div class="error-message">
          {withdrawError}
        </div>
      {/if}

      <button
        type="submit"
        class="submit-button"
        disabled={!selectedAsset || maxWithdrawable <= 0}
      >
        Withdraw
      </button>
    </form>
  </div>
</div>

<style>
  .modal-backdrop {
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
    padding: 1rem;
    box-sizing: border-box;
  }

  .modal-content {
    background: white;
    border-radius: 12px;
    width: 90%;
    max-width: 400px;
    padding: 1.5rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    max-height: 90vh;
    overflow-y: auto;
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

  .form-group input,
  .form-group select {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    box-sizing: border-box;
    background-color: white;
  }

  .form-group input:focus,
  .form-group select:focus {
    border-color: #f44336;
    outline: none;
    box-shadow: 0 0 0 2px rgba(244, 67, 54, 0.2);
  }

  .form-group select {
    cursor: pointer;
  }

  .amount-input-group {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .amount-input-group input {
    flex: 1;
  }

  .max-button {
    padding: 0.75rem 1rem;
    background-color: #ff9800;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
    white-space: nowrap;
  }

  .max-button:hover {
    background-color: #f57c00;
  }

  .balance-info {
    margin-top: 0.5rem;
    padding: 0.5rem;
    background-color: #f8f9fa;
    border-radius: 4px;
    border-left: 3px solid #f44336;
  }

  .balance-info p {
    margin: 0;
    font-size: 0.85rem;
    color: #666;
  }

  .withdraw-info {
    margin-bottom: 1rem;
    padding: 0.75rem;
    background-color: #fff3e0;
    border-radius: 6px;
    border-left: 3px solid #ff9800;
  }

  .withdraw-info p {
    margin: 0;
    font-size: 0.9rem;
    color: #666;
    word-break: break-all;
  }

  .submit-button {
    padding: 0.8rem 2rem;
    background-color: #f44336;
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    width: 100%;
    transition: background-color 0.2s;
  }

  .submit-button:hover:not(:disabled) {
    background-color: #d32f2f;
  }

  .submit-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
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

  /* Responsive adjustments */
  @media (max-width: 480px) {
    .modal-backdrop {
      padding: 0.5rem;
    }

    .modal-content {
      padding: 1rem;
    }

    .modal-header h3 {
      font-size: 1.3rem;
    }

    .amount-input-group {
      flex-direction: column;
      gap: 0.5rem;
    }

    .amount-input-group input,
    .max-button {
      width: 100%;
    }

    .submit-button {
      padding: 0.7rem 1rem;
      font-size: 0.9rem;
    }
  }
</style>
