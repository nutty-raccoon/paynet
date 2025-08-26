<script lang="ts">
  import { pushState } from "$app/navigation";
  import { type NodeData } from "../../types";
  import { formatBalance } from "../../utils";
  import { onMount, onDestroy } from "svelte";
  import AddNodeModal from "./AddNodeModal.svelte";
  import DepositModal from "./DepositModal.svelte";
  import { refresh_node_keysets } from "../../commands";

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
    refresh_node_keysets(node.id);
    selectedNodeForDeposit = node;
    // Add history entry to handle back button
    pushState("", { modal: true });
  }

  function closeDepositModal() {
    selectedNodeForDeposit = null;
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
    <div class="node-card">
      <div class="node-info">
        <div class="node-url-container">
          <span class="node-url-label">Node URL</span>
          <span class="node-url">{node.url}</span>
        </div>
        <div class="node-balance-container">
          <span class="node-balance-label">Balance</span>
          {#each node.balances as balance}
            {@const formatted = formatBalance(balance)}
            <span class="node-balance"
              >{formatted.asset}: {formatted.amount}</span
            >
          {/each}
        </div>
      </div>
      <button class="deposit-button" onclick={() => openDepositModal(node)}>
        Deposit
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
    max-width: 400px;
    gap: 1rem;
    margin: 0 auto;
    align-items: center;
  }

  .node-card {
    background-color: white;
    border-radius: 12px;
    padding: 1.25rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
    transition:
      transform 0.2s,
      box-shadow 0.2s;
  }

  .node-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .node-info {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .node-url-container,
  .node-balance-container {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .node-url-label,
  .node-balance-label {
    font-size: 0.75rem;
    text-transform: uppercase;
    color: #888;
    letter-spacing: 0.5px;
  }

  .node-url {
    font-size: 0.9rem;
    font-family: monospace;
    color: #2c3e50;
    word-break: break-all;
    padding: 0.375rem 0.5rem;
    background-color: #f8f9fa;
    border-radius: 4px;
  }

  .node-balance {
    font-size: 1.5rem;
    font-weight: 600;
    color: #1e88e5;
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

  .deposit-button {
    margin-top: 0.75rem;
    padding: 0.5rem 1rem;
    background-color: #4caf50;
    color: white;
    font-weight: 500;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background-color 0.2s;
  }

  .deposit-button:hover {
    background-color: #45a049;
  }
</style>
