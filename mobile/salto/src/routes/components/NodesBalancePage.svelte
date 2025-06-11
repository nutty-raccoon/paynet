<script lang="ts">
  import type { EventHandler } from "svelte/elements";
  import { type NodeData } from "../../types";
  import { formatBalance } from "../../utils";
  import { onMount, onDestroy } from "svelte";
  import { addNode } from "../../commands";

  interface Props {
    nodes: NodeData[];
    onAddNode: (nodeData: NodeData) => void;
  }

  let { nodes, onAddNode }: Props = $props();

  // Modal state
  let isAddNodeModalOpen = $state(false);
  let selectedNodeForDeposit = $state<NodeData | null>(null);
  let depositError = $state<string>("");

  // Show modal and push state to history
  function openAddNodeModal() {
    isAddNodeModalOpen = true;
    // Add history entry to handle back button
    history.pushState({ modal: true }, "");
  }

  // Hide modal
  function closeAddNodeModal() {
    isAddNodeModalOpen = false;
  }

  // Show deposit modal
  function openDepositModal(node: NodeData) {
    selectedNodeForDeposit = node;
    depositError = "";
    // Add history entry to handle back button
    history.pushState({ modal: true }, "");
  }

  // Hide deposit modal
  function closeDepositModal() {
    selectedNodeForDeposit = null;
    depositError = "";
  }

  const handleAddNodeFormSubmit: EventHandler<SubmitEvent, HTMLFormElement> = (
    event,
  ) => {
    event.preventDefault();
    const form = event.target as HTMLFormElement;
    const formDataObject = new FormData(form);
    const nodeAddress = formDataObject.get("node-address");
    if (!!nodeAddress) {
      let nodeAddressString = nodeAddress.toString();
      addNode(nodeAddressString).then((newNodeData) => {
        if (!!newNodeData) {
          const nodeId = newNodeData[0];
          // Check if node with this ID already exists in the nodes array
          const nodeAlreadyListed = nodes.some((node) => node.id === nodeId);
          if (!nodeAlreadyListed) {
            onAddNode({
              id: nodeId,
              url: nodeAddressString,
              balances: newNodeData[1],
            });
          } else {
            console.log(`node with url ${nodeAddress} already declared`);
          }
        }
      });
    }
    closeAddNodeModal();
  };

  const handleDepositFormSubmit: EventHandler<SubmitEvent, HTMLFormElement> = (
    event,
  ) => {
    event.preventDefault();
    const form = event.target as HTMLFormElement;
    const formDataObject = new FormData(form);
    const amount = formDataObject.get("deposit-amount");
    const token = formDataObject.get("deposit-token");

    // Clear previous error
    depositError = "";

    if (selectedNodeForDeposit && amount && token) {
      const amountValue = parseFloat(amount.toString());

      if (amountValue <= 0) {
        depositError = "Amount must be greater than 0";
        return;
      }

      console.log(
        `Depositing ${amountValue} ${token} to node ${selectedNodeForDeposit.url}`,
      );
      // TODO: Implement actual deposit logic here
      // This would typically call a Tauri command to handle the deposit

      closeDepositModal();
    }
  };

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
            <span class="node-balance">{formatBalance(balance)}</span>
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

<!-- Add Node Modal overlay -->
{#if isAddNodeModalOpen}
  <div class="modal-overlay">
    <div class="modal-content">
      <div class="modal-header">
        <h3>Add Node</h3>
        <button class="close-button" onclick={closeAddNodeModal}>✕</button>
      </div>

      <form onsubmit={handleAddNodeFormSubmit}>
        <div class="form-group">
          <label for="node-address">Node's address</label>
          <input
            type="url"
            id="node-address"
            name="node-address"
            placeholder="https://example.com"
            required
          />
        </div>

        <button type="submit" class="add-button">Add</button>
      </form>
    </div>
  </div>
{/if}

<!-- Deposit Modal overlay -->
{#if !!selectedNodeForDeposit}
  <div class="modal-overlay">
    <div class="modal-content">
      <div class="modal-header">
        <h3>Deposit Tokens</h3>
        <button class="close-button" onclick={closeDepositModal}>✕</button>
      </div>

      <form onsubmit={handleDepositFormSubmit}>
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
          <p>Depositing to: {selectedNodeForDeposit.url}</p>
        </div>

        {#if depositError}
          <div class="error-message">
            {depositError}
          </div>
        {/if}

        <button type="submit" class="deposit-submit-button">Deposit</button>
      </form>
    </div>
  </div>
{/if}

<style>
  .nodes-container {
    display: flex;
    flex-direction: column;
    width: 80%;
    max-width: 400px;
    gap: 1rem;
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
  }

  .add-node-button:hover {
    background-color: #1976d2;
  }

  /* Modal styles */
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

  .add-button {
    padding: 0.8rem 2rem;
    background-color: #1e88e5;
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    width: 100%;
    transition: background-color 0.2s;
  }

  .add-button:hover {
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

  .deposit-submit-button {
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

  .deposit-submit-button:hover {
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
