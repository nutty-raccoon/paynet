<script lang="ts">
  import type { NodeId, Balance, NodeIdAndUrl } from "../../types";
  import { pendingQuotes } from "../../stores";
  import { derived as derivedStore } from "svelte/store";
  import DepositModal from "./DepositModal.svelte";
  import WithdrawModal from "./WithdrawModal.svelte";
  import { formatBalance } from "../../utils";
  import { t } from "../../stores/i18n";
  import {
    payMeltQuote,
    payMintQuote,
    redeemQuote,
    forgetNode,
  } from "../../commands";
  import type { QuoteId } from "../../types/quote";
  import type { MintSettings } from "../../types/NodeMintMethodInfo";

  interface Props {
    selectedNode: NodeIdAndUrl;
    nodeBalances: Balance[];
    nodeDepositSettings: MintSettings | null;
    onClose: () => void;
  }

  let { selectedNode, nodeBalances, nodeDepositSettings, onClose }: Props =
    $props();

  type ModalState = "none" | "deposit" | "withdraw";
  let currentModal = $state<ModalState>("none");

  // Get pending quotes for this specific node
  const nodePendingQuotes = derivedStore(pendingQuotes, ($pendingQuotes) => {
    const nodeQuotes = $pendingQuotes.get(selectedNode.id);
    return {
      mint: nodeQuotes?.mint || { unpaid: [], paid: [] },
      melt: nodeQuotes?.melt || { unpaid: [], pending: [] },
    };
  });

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  function openDepositModal() {
    if (!!nodeDepositSettings && !nodeDepositSettings.disabled) {
      currentModal = "deposit";
    } else {
      // TODO: display error message saying deposits are not available for this node
    }
  }

  function openWithdrawModal() {
    currentModal = "withdraw";
  }

  function closeModal() {
    currentModal = "none";
  }

  function handleUnpaidQuotePay(nodeId: NodeId, quoteId: QuoteId) {
    payMintQuote(nodeId, quoteId);
  }

  function handlePaidQuotePay(nodeId: NodeId, quoteId: QuoteId) {
    redeemQuote(nodeId, quoteId);
  }

  function handleMeltUnpaidQuotePay(nodeId: NodeId, quoteId: QuoteId) {
    payMeltQuote(nodeId, quoteId);
  }

  // Check if node should show forget button (no balances and no pending quotes)
  const shouldShowForgetButton = $derived.by(() => {
    const hasBalances = nodeBalances.length > 0;
    const hasMintQuotes =
      $nodePendingQuotes.mint.unpaid.length > 0 ||
      $nodePendingQuotes.mint.paid.length > 0;
    const hasMeltQuotes =
      $nodePendingQuotes.melt.unpaid.length > 0 ||
      $nodePendingQuotes.melt.pending.length > 0;

    return !hasBalances && !hasMintQuotes && !hasMeltQuotes;
  });

  function handleForgetNode() {
    forgetNode(selectedNode.id, false);
    onClose();
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
        <h3 class="section-title">{$t("balance.balances")}</h3>
        {#if nodeBalances.length > 0}
          <div class="balances-list">
            {#each nodeBalances as balance}
              {@const formatted = formatBalance(balance.unit, balance.amount)}
              <div class="balance-item">
                <span class="quote-amount"
                  >{formatted.assetAmount} {formatted.asset}</span
                >
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty-state">{$t("balance.noBalancesAvailable")}</p>
        {/if}
      </div>

      <!-- Pending Mint Quotes Section -->
      <div class="section">
        <h3 class="section-title">{$t("balance.pendingMintQuotes")}</h3>
        {#if $nodePendingQuotes.mint.unpaid.length > 0 || $nodePendingQuotes.mint.paid.length > 0}
          {#if $nodePendingQuotes.mint.unpaid.length > 0}
            <div class="quotes-subsection">
              <h4 class="subsection-title">
                {$t("balance.unpaid")} ({$nodePendingQuotes.mint.unpaid.length})
              </h4>
              <div class="quotes-list">
                {#each $nodePendingQuotes.mint.unpaid as quote}
                  {@const formatted = formatBalance(quote.unit, quote.amount)}
                  <div class="quote-item pending">
                    <div class="quote-info">
                      <span class="quote-amount"
                        >{formatted.assetAmount} {formatted.asset}</span
                      >
                    </div>
                    <button
                      class="pay-button pending"
                      onclick={() =>
                        handleUnpaidQuotePay(selectedNode.id, quote.id)}
                    >
                      {$t("balance.pay")}
                    </button>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          {#if $nodePendingQuotes.mint.paid.length > 0}
            <div class="quotes-subsection">
              <h4 class="subsection-title">
                {$t("balance.paid")} ({$nodePendingQuotes.mint.paid.length})
              </h4>
              <div class="quotes-list">
                {#each $nodePendingQuotes.mint.paid as quote}
                  {@const formatted = formatBalance(quote.unit, quote.amount)}
                  <div class="quote-item pending">
                    <div class="quote-info">
                      <span class="quote-amount"
                        >{formatted.assetAmount} {formatted.asset}</span
                      >
                    </div>
                    <button
                      class="pay-button pending"
                      onclick={() =>
                        handlePaidQuotePay(selectedNode.id, quote.id)}
                    >
                      {$t("balance.redeem")}
                    </button>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        {:else}
          <p class="empty-state">{$t("balance.noPendingMintQuotes")}</p>
        {/if}
      </div>

      <!-- Pending Melt Quotes Section -->
      <div class="section">
        <h3 class="section-title">{$t("balance.pendingMeltQuotes")}</h3>
        {#if $nodePendingQuotes.melt.unpaid.length > 0 || $nodePendingQuotes.melt.pending.length > 0}
          {#if $nodePendingQuotes.melt.unpaid.length > 0}
            <div class="quotes-subsection">
              <h4 class="subsection-title">
                {$t("balance.unpaid")} ({$nodePendingQuotes.melt.unpaid.length})
              </h4>
              <div class="quotes-list">
                {#each $nodePendingQuotes.melt.unpaid as quote}
                  {@const formatted = formatBalance(quote.unit, quote.amount)}
                  <div class="quote-item pending">
                    <div class="quote-info">
                      <span class="quote-amount"
                        >{formatted.assetAmount} {formatted.asset}</span
                      >
                    </div>
                    <button
                      class="pay-button pending"
                      onclick={() =>
                        handleMeltUnpaidQuotePay(selectedNode.id, quote.id)}
                    >
                      {$t("balance.pay")}
                    </button>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          {#if $nodePendingQuotes.melt.pending.length > 0}
            <div class="quotes-subsection">
              <h4 class="subsection-title">
                {$t("balance.pending")} ({$nodePendingQuotes.melt.pending
                  .length})
              </h4>
              <div class="quotes-list">
                {#each $nodePendingQuotes.melt.pending as quote}
                  {@const formatted = formatBalance(quote.unit, quote.amount)}
                  <div class="quote-item pending">
                    <div class="quote-info">
                      <span class="quote-amount"
                        >{formatted.assetAmount} {formatted.asset}</span
                      >
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        {:else}
          <p class="empty-state">{$t("balance.noPendingMeltQuotes")}</p>
        {/if}
      </div>
    </div>

    <div class="modal-footer">
      <div class="action-buttons">
        <button
          class="deposit-button"
          class:disabled={!nodeDepositSettings || nodeDepositSettings.disabled}
          disabled={!nodeDepositSettings || nodeDepositSettings.disabled}
          onclick={openDepositModal}
          title={!nodeDepositSettings
            ? $t("validation.noDepositMethodsAvailable")
            : nodeDepositSettings.disabled
              ? $t("validation.depositsDisabledForNode")
              : ""}
        >
          {$t("modals.deposit")}
        </button>
        {#if shouldShowForgetButton}
          <button class="forget-button" onclick={handleForgetNode}>
            {$t("balance.forget")}
          </button>
        {:else}
          <button class="withdraw-button" onclick={openWithdrawModal}>
            {$t("modals.withdraw")}
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

{#if currentModal === "deposit"}
  {#if !!nodeDepositSettings}
    <DepositModal {selectedNode} onClose={closeModal} {nodeDepositSettings} />
  {:else}
    <div>{$t("validation.errorNoDepositSettings")}</div>
  {/if}
{:else if currentModal === "withdraw"}
  <WithdrawModal {selectedNode} {nodeBalances} onClose={closeModal} />
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
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .quote-item.pending {
    background-color: #fff3e0;
    border-left-color: #ff9800;
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

  .pay-button {
    padding: 0.4rem 0.8rem;
    border: none;
    border-radius: 6px;
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    min-width: 60px;
  }

  .pay-button.pending {
    background-color: #ff9800;
    color: white;
  }

  .pay-button.pending:hover {
    background-color: #f57c00;
    transform: translateY(-1px);
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

  .action-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .action-buttons button {
    flex: 1;
  }

  .deposit-button {
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

  .deposit-button.disabled {
    background-color: #cccccc;
    color: #666666;
    cursor: not-allowed;
  }

  .deposit-button.disabled:hover {
    background-color: #cccccc;
  }

  .withdraw-button {
    padding: 0.8rem 1rem;
    background-color: #f44336;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .withdraw-button:hover {
    background-color: #d32f2f;
  }

  .forget-button {
    padding: 0.8rem 1rem;
    background-color: #f44336;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .forget-button:hover {
    background-color: #d32f2f;
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

    .action-buttons button {
      padding: 0.7rem 1rem;
      font-size: 0.9rem;
    }

    .pay-button {
      padding: 0.3rem 0.6rem;
      font-size: 0.75rem;
      min-width: 50px;
    }
  }
</style>
