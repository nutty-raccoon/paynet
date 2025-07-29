<script lang="ts">
  import type { EventHandler } from "svelte/elements";
  import type { NodeData } from "../../types";
  import { create_mint_quote, redeem_quote } from "../../commands";

  interface Props {
    selectedNode: NodeData;
    onClose: () => void;
  }

  let { selectedNode, onClose }: Props = $props();
  let depositError = $state<string>("");

  const handleFormSubmit: EventHandler<SubmitEvent, HTMLFormElement> = (
    event,
  ) => {
    event.preventDefault();
    const form = event.target as HTMLFormElement;
    const formDataObject = new FormData(form);
    const token = formDataObject.get("deposit-token");
    const amount = formDataObject.get("deposit-amount");

    // Clear previous error
    depositError = "";

    if (selectedNode && amount && token) {
      const amountString = amount.toString();
      const amountValue = parseFloat(amountString);
      const nodeId = selectedNode["id"];

      if (amountValue <= 0) {
        depositError = "Amount must be greater than 0";
        return;
      }

      create_mint_quote(nodeId, amountString, token.toString()).then(
        (createMintQuoteResponse) => {
          if (!!createMintQuoteResponse) {
            if (createMintQuoteResponse.paymentRequest.length === 0) {
              redeem_quote(nodeId, createMintQuoteResponse.quoteId);
            } else {
              console.log("todo: proceed to payment");
            }
          }
        },
      );

      onClose();
    }
  };

  // Reset error when modal closes
  $effect(() => {
    if (!selectedNode) {
      depositError = "";
    }
  });
</script>

<div class="modal-overlay">
  <div class="modal-content">
    <div class="modal-header">
      <h3>Deposit Tokens</h3>
      <button class="close-button" onclick={onClose}>✕</button>
    </div>

    <form onsubmit={handleFormSubmit}>
      <div class="form-group">
        <label for="deposit-amount">Amount</label>
        <div class="amount-input-group">
          <input
            type="number"
            id="deposit-amount"
            name="deposit-amount"
            placeholder="0.0"
            min="0"
            step="any"
            required
          />
          <select name="deposit-token" required>
            <option value="strk">STRK</option>
            <option value="eth">ETH</option>
          </select>
        </div>
      </div>

      <div class="deposit-info">
        <p>Depositing to: {selectedNode.url}</p>
      </div>

      {#if depositError}
        <div class="error-message">
          {depositError}
        </div>
      {/if}

      <button type="submit" class="submit-button">Deposit</button>
    </form>
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

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-group label {
    display: block;
    font-size: 0.9rem;
    margin-bottom: 0.5rem;
    color: #333;
  }

  .form-group input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    box-sizing: border-box;
  }

  .form-group input:focus {
    border-color: #1e88e5;
    outline: none;
    box-shadow: 0 0 0 2px rgba(30, 136, 229, 0.2);
  }

  .amount-input-group {
    display: flex;
    gap: 0.5rem;
  }

  .amount-input-group input {
    flex: 2;
  }

  .amount-input-group select {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    background-color: white;
    cursor: pointer;
  }

  .amount-input-group select:focus {
    border-color: #1e88e5;
    outline: none;
    box-shadow: 0 0 0 2px rgba(30, 136, 229, 0.2);
  }

  .deposit-info {
    margin-bottom: 1rem;
    padding: 0.75rem;
    background-color: #f8f9fa;
    border-radius: 6px;
    border-left: 3px solid #1e88e5;
  }

  .deposit-info p {
    margin: 0;
    font-size: 0.9rem;
    color: #666;
    word-break: break-all;
  }

  .submit-button {
    padding: 0.8rem 2rem;
    background-color: #4caf50;
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    width: 100%;
    transition: background-color 0.2s;
  }

  .submit-button:hover {
    background-color: #45a049;
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
