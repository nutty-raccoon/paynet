<script lang="ts">
  import { type Node } from "../../types";
  import { formatBalance } from "../../utils";

  interface Props {
    nodes: Node[];
  }

  let { nodes }: Props = $props();
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
          <span class="node-balance">{formatBalance(node.balance)}</span>
        </div>
      </div>
    </div>
  {/each}
</div>

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
</style>
