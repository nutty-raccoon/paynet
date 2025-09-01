<script lang="ts">
  import type { NodeData } from "../../types";
  import { pendingMintQuotes } from "../../stores";
  import { derived } from "svelte/store";
  import DepositModal from "./DepositModal.svelte";

  interface Props {
    selectedNode: NodeData;
    onClose: () => void;
  }

  let { selectedNode, onClose }: Props = $props();
  let showDepositModal = $state(false);

  // Get pending quotes for this specific node
  const nodePendingQuotes = derived(pendingMintQuotes, ($pendingMintQuotes) => {
    return $pendingMintQuotes.get(selectedNode.id) || { unpaid: [], paid: [] };
  });

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  function openDepositModal() {
    showDepositModal = true;
  }

  function closeDepositModal() {
    showDepositModal = false;
  }
</script>

<div class="modal-backdrop" onclick={handleBackdropClick}>
  <div class="modal-content">
    <div class="modal-header">
      <h2 class="node-url">{selectedNode.url}</h2>
      <button class="close-button" onclick={onClose}>×</button>
    </div>

    <div class="modal-body">
      <!-- Balances Section -->
      <div class="section">
        <h3 class="section-title">Balances</h3>
        {#if selectedNode.balances.length > 0}
          <div class="balances-list">
            {#each selectedNode.balances as balance}
              <div class="balance-item">
                <span class="balance-unit">{balance.unit}</span>
                <span class="balance-amount">{balance.amount}</span>
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty-state">No balances available</p>
        {/if}
      </div>

      <!-- Pending Mint Quotes Section -->
      <div class="section">
        <h3 class="section-title">Pending Mint Quotes</h3>
        {#if $nodePendingQuotes.unpaid.length > 0 || $nodePendingQuotes.paid.length > 0}
          {#if $nodePendingQuotes.unpaid.length > 0}
            <div class="quotes-subsection">
              <h4 class="subsection-title">
                Unpaid ({$nodePendingQuotes.unpaid.length})
              </h4>
              <div class="quotes-list">
                {#each $nodePendingQuotes.unpaid as quote}
                  <div class="quote-item unpaid">
                    <div class="quote-info">
                      <span class="quote-amount"
                        >{quote.amount} {quote.unit}</span
                      >
                      <span class="quote-id">{quote.quote}</span>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          {#if $nodePendingQuotes.paid.length > 0}
            <div class="quotes-subsection">
              <h4 class="subsection-title">
                Paid ({$nodePendingQuotes.paid.length})
              </h4>
              <div class="quotes-list">
                {#each $nodePendingQuotes.paid as quote}
                  <div class="quote-item paid">
                    <div class="quote-info">
                      <span class="quote-amount"
                        >{quote.amount} {quote.unit}</span
                      >
                      <span class="quote-id">{quote.quote}</span>
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        {:else}
          <p class="empty-state">No pending mint quotes</p>
        {/if}
      </div>
    </div>

    <div class="modal-footer">
      <button class="deposit-button" onclick={openDepositModal}>
        Deposit
      </button>
    </div>
  </div>
</div>

{#if showDepositModal}
  <DepositModal selectedNode={selectedNode} onClose={closeDepositModal} />
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
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
    width: 100%;
    max-width: 500px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
  }

  .modal-header {
    padding: 1.5rem;
    border-bottom: 1px solid #e0e0e0;
    display: flex;
    justify-content: space-between;
    align-items: center;
    background-color: #f8f9fa;
  }

  .node-url {
    font-size: 1.1rem;
    font-weight: 600;
    color: #2c3e50;
    font-family: monospace;
    margin: 0;
    word-break: break-all;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 2rem;
    cursor: pointer;
    color: #666;
    padding: 0;
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition: background-color 0.2s;
  }

  .close-button:hover {
    background-color: #e0e0e0;
  }

  .modal-body {
    padding: 1.5rem;
    overflow-y: auto;
    flex: 1;
  }

  .section {
    margin-bottom: 2rem;
  }

  .section:last-child {
    margin-bottom: 0;
  }

  .section-title {
    font-size: 1.2rem;
    font-weight: 600;
    color: #2c3e50;
    margin-bottom: 1rem;
    margin-top: 0;
  }

  .balances-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .balance-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    background-color: #f8f9fa;
    border-radius: 8px;
    border-left: 4px solid #1e88e5;
  }

  .balance-unit {
    font-weight: 500;
    color: #2c3e50;
    font-family: monospace;
  }

  .balance-amount {
    font-weight: 600;
    color: #1e88e5;
    font-size: 1.1rem;
  }

  .quotes-subsection {
    margin-bottom: 1.5rem;
  }

  .subsection-title {
    font-size: 1rem;
    font-weight: 500;
    color: #666;
    margin-bottom: 0.75rem;
    margin-top: 0;
  }

  .quotes-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .quote-item {
    padding: 0.75rem;
    border-radius: 8px;
    border-left: 4px solid;
  }

  .quote-item.unpaid {
    background-color: #fff3e0;
    border-left-color: #ff9800;
  }

  .quote-item.paid {
    background-color: #e8f5e8;
    border-left-color: #4caf50;
  }

  .quote-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .quote-amount {
    font-weight: 600;
    color: #2c3e50;
  }

  .quote-id {
    font-size: 0.8rem;
    font-family: monospace;
    color: #666;
    word-break: break-all;
  }

  .empty-state {
    color: #999;
    font-style: italic;
    text-align: center;
    padding: 2rem;
    margin: 0;
  }

  .modal-footer {
    padding: 1.5rem;
    border-top: 1px solid #e0e0e0;
    background-color: #f8f9fa;
  }

  .deposit-button {
    width: 100%;
    padding: 0.8rem 1rem;
    background-color: #4caf50;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .deposit-button:hover {
    background-color: #45a049;
  }

  /* Responsive adjustments */
  @media (max-width: 480px) {
    .modal-backdrop {
      padding: 0.5rem;
    }

    .modal-header {
      padding: 1rem;
    }

    .modal-body {
      padding: 1rem;
    }

    .node-url {
      font-size: 0.9rem;
    }

    .balance-item,
    .quote-item {
      padding: 0.5rem;
    }

    .modal-footer {
      padding: 1rem;
    }

    .deposit-button {
      padding: 0.7rem 1rem;
      font-size: 0.9rem;
    }
  }
</style>
