<script lang="ts">
  import { onMount } from "svelte";
  import { get_wad_history } from "../../commands";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  interface WadHistoryItem {
    id: number;
    wadType: string;
    status: string;
    totalAmountJson: string;
    memo?: string;
    createdAt: number;
    modifiedAt: number;
  }

  let wadHistory: WadHistoryItem[] = $state([]);
  let loading = $state(true);
  let error = $state("");

  onMount(async () => {
    await loadWadHistory();
  });

  async function loadWadHistory() {
    try {
      loading = true;
      error = "";
      
      console.log("Fetching WAD history...");
      const history = await get_wad_history(20);
      console.log("WAD history received:", history);
      
      wadHistory = history || [];
    } catch (err) {
      console.error("Failed to load WAD history:", err);
      error = "Failed to load WAD history: " + String(err);
    } finally {
      loading = false;
    }
  }

  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  function formatAmount(amountJson: string): string {
    try {
      const parsed = JSON.parse(amountJson);
      if (Array.isArray(parsed) && parsed.length > 0) {
        const [unit, amount] = Object.entries(parsed[0])[0];
        return `${amount} ${unit}`;
      }
    } catch (e) {
      console.error("Error parsing amount:", e);
    }
    return amountJson;
  }

  function getStatusColor(status: string): string {
    switch (status.toLowerCase()) {
      case "finished": return "#28a745";
      case "pending": return "#ffc107";
      case "failed": return "#dc3545";
      case "cancelled": return "#6c757d";
      default: return "#007bff";
    }
  }

  function getTypeIcon(type: string): string {
    return type.toLowerCase() === "incoming" ? "📥" : "📤";
  }
</script>

<div class="modal-overlay" 
     role="dialog" 
     aria-modal="true"
     tabindex="-1"
     onclick={onClose} 
     onkeydown={(e) => e.key === 'Escape' && onClose()}>
  <div class="modal-content" 
       role="document"
       onclick={(e) => e.stopPropagation()}
       onkeydown={(e) => e.stopPropagation()}>
    <div class="modal-header">
      <h2>WAD History</h2>
      <button class="close-btn" onclick={onClose}>✕</button>
    </div>

    <div class="modal-body">
      {#if loading}
        <div class="loading">
          <div class="spinner"></div>
          <p>Loading WAD history...</p>
        </div>
      {:else if error}
        <div class="error">
          <p>{error}</p>
          <button onclick={loadWadHistory}>Retry</button>
        </div>
      {:else if wadHistory.length === 0}
        <div class="empty">
          <p>No WAD history found</p>
        </div>
      {:else}
        <div class="history-list">
          {#each wadHistory as wad}
            <div class="wad-item">
              <div class="wad-header">
                <div class="wad-type">
                  <span class="type-icon">{getTypeIcon(wad.wadType)}</span>
                  <span class="type-text">{wad.wadType}</span>
                </div>
                <div class="wad-status" style="color: {getStatusColor(wad.status)}">
                  {wad.status}
                </div>
              </div>
              
              <div class="wad-amount">
                {formatAmount(wad.totalAmountJson)}
              </div>
              
              {#if wad.memo}
                <div class="wad-memo">
                  📝 {wad.memo}
                </div>
              {/if}
              
              <div class="wad-details">
                                            <div class="wad-id">ID: {wad.id}</div>
                <div class="wad-time">
                  Created: {formatTimestamp(wad.createdAt)}
                </div>
                {#if wad.modifiedAt !== wad.createdAt}
                  <div class="wad-time">
                    Modified: {formatTimestamp(wad.modifiedAt)}
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="modal-footer">
      <button onclick={loadWadHistory} disabled={loading}>
        🔄 Refresh
      </button>
    </div>
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
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 20px;
  }

  .modal-content {
    background: white;
    border-radius: 12px;
    max-width: 500px;
    width: 100%;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    border-bottom: 1px solid #eee;
  }

  .modal-header h2 {
    margin: 0;
    color: #333;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
    color: #666;
    padding: 0;
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn:hover {
    color: #333;
  }

  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .loading, .error, .empty {
    text-align: center;
    padding: 40px 20px;
  }

  .spinner {
    border: 3px solid #f3f3f3;
    border-top: 3px solid #007bff;
    border-radius: 50%;
    width: 30px;
    height: 30px;
    animation: spin 1s linear infinite;
    margin: 0 auto 15px;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .wad-item {
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    padding: 15px;
    background: #fafafa;
  }

  .wad-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }

  .wad-type {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .type-icon {
    font-size: 18px;
  }

  .type-text {
    font-weight: 600;
    text-transform: capitalize;
  }

  .wad-status {
    font-weight: 600;
    font-size: 14px;
    text-transform: uppercase;
  }

  .wad-amount {
    font-size: 18px;
    font-weight: 700;
    color: #2c5aa0;
    margin-bottom: 10px;
  }

  .wad-memo {
    font-style: italic;
    color: #666;
    margin-bottom: 10px;
    padding: 8px;
    background: white;
    border-radius: 4px;
    border-left: 3px solid #007bff;
  }

  .wad-details {
    font-size: 12px;
    color: #666;
    line-height: 1.4;
  }

  .wad-uuid {
    font-family: monospace;
    margin-bottom: 4px;
  }

  .modal-footer {
    padding: 20px;
    border-top: 1px solid #eee;
    display: flex;
    justify-content: center;
  }

  .modal-footer button {
    background: #007bff;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  .modal-footer button:hover:not(:disabled) {
    background: #0056b3;
  }

  .modal-footer button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error button {
    background: #dc3545;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    margin-top: 10px;
  }

  .error button:hover {
    background: #c82333;
  }
</style> 