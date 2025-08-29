<script lang="ts">
  import { pushState } from "$app/navigation";
  import { type NodeData } from "../../types";
  import {
    formatBalance,
    computeTotalBalancePerUnit,
    getTotalAmountInDisplayCurrency,
  } from "../../utils";
  import { tokenPrices, displayCurrency } from "../../stores";
  import { onMount, onDestroy } from "svelte";
  import AddNodeModal from "./AddNodeModal.svelte";
  import DepositModal from "./DepositModal.svelte";
  import { refreshNodeKeysets } from "../../commands";

  interface Props {
    nodes: NodeData[];
    onAddNode: (nodeData: NodeData) => void;
  }

  let { nodes, onAddNode }: Props = $props();

  // Modal state
  let isAddNodeModalOpen = $state(false);
  let selectedNodeForDeposit = $state<NodeData | null>(null);

  // Modal control functions
  function openAddNodeModal() {
    isAddNodeModalOpen = true;
    // Add history entry to handle back button
    pushState("", { modal: true });
  }

  function closeAddNodeModal() {
    isAddNodeModalOpen = false;
  }

  function openDepositModal(node: NodeData) {
    refreshNodeKeysets(node.id);
    selectedNodeForDeposit = node;
    // Add history entry to handle back button
    pushState("", { modal: true });
  }

  function closeDepositModal() {
    selectedNodeForDeposit = null;
  }

  // Function to compute total balance for a single node
  function getNodeTotalBalance(node: NodeData): string {
    const nodeBalanceMap = new Map();

    // Convert node balances to the same format as computeTotalBalancePerUnit expects
    node.balances.forEach((balance) => {
      nodeBalanceMap.set(balance.unit, balance.amount);
    });

    if (!!$tokenPrices) {
      const totalValue = getTotalAmountInDisplayCurrency(
        nodeBalanceMap,
        $tokenPrices,
      );
      return `${totalValue.toFixed(2)} ${$displayCurrency}`;
    } else {
      return `??? ${$displayCurrency}`;
    }
  }

  // Set up back button listener
  function handlePopState() {
    if (!!selectedNodeForDeposit) {
      closeDepositModal();
    } else if (isAddNodeModalOpen) {
      closeAddNodeModal();
    }
  }

  onMount(() => {
    window.addEventListener("popstate", handlePopState);
  });

  onDestroy(() => {
    window.removeEventListener("popstate", handlePopState);
  });
</script>

<div class="nodes-container">
  {#each nodes as node}
    <div class="node-row">
      <div class="node-info">
        <span class="node-url">{node.url}</span>
        <span class="node-balance">{getNodeTotalBalance(node)}</span>
      </div>
      <button class="open-button" onclick={() => openDepositModal(node)}>
        Open
      </button>
    </div>
  {/each}

  <button class="add-node-button" onclick={openAddNodeModal}> Add Node </button>
</div>

{#if isAddNodeModalOpen}
  <AddNodeModal {nodes} onClose={closeAddNodeModal} {onAddNode} />
{/if}

{#if !!selectedNodeForDeposit}
  <DepositModal
    selectedNode={selectedNodeForDeposit}
    onClose={closeDepositModal}
  />
{/if}

<style>
  .nodes-container {
    display: flex;
    flex-direction: column;
    width: 90%;
    max-width: 600px;
    gap: 0.5rem;
    margin: 0 auto;
    align-items: center;
  }

  .node-row {
    background-color: white;
    border-radius: 8px;
    padding: 1rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
    transition:
      transform 0.2s,
      box-shadow 0.2s;
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    box-sizing: border-box;
  }

  .node-row:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .node-info {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    flex: 1;
    margin-right: 1rem;
  }

  .node-url {
    font-size: 0.9rem;
    font-family: monospace;
    color: #2c3e50;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .node-balance {
    font-size: 1.25rem;
    font-weight: 600;
    color: #1e88e5;
  }

  .open-button {
    padding: 0.6rem 1.2rem;
    background-color: #4caf50;
    color: white;
    font-weight: 500;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background-color 0.2s;
    flex-shrink: 0;
  }

  .open-button:hover {
    background-color: #45a049;
  }

  .add-node-button {
    margin-top: 1rem;
    padding: 0.8rem 1.5rem;
    background-color: #1e88e5;
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background-color 0.2s;
    width: 100%;
    box-sizing: border-box;
  }

  .add-node-button:hover {
    background-color: #1976d2;
  }

  /* Responsive adjustments for smaller screens */
  @media (max-width: 480px) {
    .node-row {
      padding: 0.75rem;
    }

    .node-url {
      font-size: 0.8rem;
    }

    .node-balance {
      font-size: 1.1rem;
    }

    .open-button {
      padding: 0.5rem 1rem;
      font-size: 0.85rem;
    }
  }
</style>
