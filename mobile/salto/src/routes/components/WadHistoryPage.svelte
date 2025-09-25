<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getWadHistory, syncWads } from "../../commands";
  import { formatBalance } from "../../utils";
  import type { WadHistoryItem, WadStatus } from "../../types/wad";
  import type { Balance } from "../../types";
  import { t } from "../../stores/i18n";

  let wadHistory: WadHistoryItem[] = $state([]);
  let loading = $state(true);
  let syncing = $state(false);
  let error = $state("");
  let expandedWadId: [string, string] | null = $state(null);

  // Store unsubscribe functions for cleanup
  let unsubscribeWadStatusUpdated: (() => void) | null = null;
  let unsubscribeSyncError: (() => void) | null = null;

  onMount(async () => {
    unsubscribeWadStatusUpdated = await listen<{
      wadId: string;
      newStatus: string;
    }>("wad-status-updated", (event) => {
      wadHistory = wadHistory.map((wad) =>
        wad.id === event.payload.wadId
          ? { ...wad, status: event.payload.newStatus as WadStatus }
          : wad,
      );
    });
    unsubscribeSyncError = await listen<{ wadId: string; error: string }>(
      "sync-wad-error",
      (event) => {
        console.error(
          `Sync error for WAD ${event.payload.wadId}:`,
          event.payload.error,
        );
      },
    );

    await loadWadHistory();
  });

  onDestroy(() => {
    if (unsubscribeWadStatusUpdated) {
      unsubscribeWadStatusUpdated();
    }
    if (unsubscribeSyncError) {
      unsubscribeSyncError();
    }
  });

  async function loadWadHistory() {
    try {
      loading = true;
      error = "";

      // Show cached data immediately
      const history = await getWadHistory(20);
      wadHistory = history || [];
      loading = false; // UI updates with cached data

      // Then sync in background
      syncing = true;
      await syncWads();
    } catch (err) {
      // handle error
    } finally {
      syncing = false;
      loading = false;
    }
  }

  function formatTimestamp(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  function formatAmount(amounts: Balance[]): string {
    try {
      // TODO: handle wads with more than 1 asset
      const { asset, assetAmount } = formatBalance(
        amounts[0].unit,
        amounts[0].amount,
      );
      return `${assetAmount} ${asset}`;
    } catch (e) {
      console.log(`failed to format amount: ${e}`);
      return "";
    }
  }

  function getStatusColor(status: string): string {
    switch (status.toLowerCase()) {
      case "finished":
        return "#28a745";
      case "pending":
        return "#ffc107";
      case "failed":
        return "#dc3545";
      case "cancelled":
        return "#6c757d";
      default:
        return "#007bff";
    }
  }

  function getTypeIcon(type: string): string {
    return type.toLowerCase() === "in" ? "📥" : "📤";
  }

  function getTypeDisplay(type: string): string {
    return type.toLowerCase() === "in" ? $t('history.in') : $t('history.out');
  }

  function toggleWadExpansion(wadId: string, wadType: string) {
    if (!expandedWadId) {
      expandedWadId = [wadId, wadType];
    } else if (expandedWadId[0] == wadId && expandedWadId[1] == wadType) {
      expandedWadId = null;
    } else {
      expandedWadId = [wadId, wadType];
    }
  }

  function isWadSelected(wadId: string, wadType: string) {
    return (
      !!expandedWadId &&
      expandedWadId[0] == wadId &&
      expandedWadId[1] == wadType
    );
  }
</script>

<div class="history-page">
  <div class="history-container">
    <div class="header">
      <h1>{$t('history.transferHistory')}</h1>
    </div>

    <div class="content">
      {#if loading}
        <div class="loading">
          <div class="spinner"></div>
          <p>{$t('history.loadingHistory')}</p>
        </div>
      {:else if error}
        <div class="error">
          <p>{error}</p>
          <button onclick={loadWadHistory}>{$t('forms.retry')}</button>
        </div>
      {:else if wadHistory.length === 0}
        <div class="empty">
          <p>{$t('history.noTransactions')}</p>
        </div>
      {:else}
        <div class="history-list">
          {#each wadHistory as wad}
            <div class="wad-item">
              <button
                class="wad-line clickable"
                onclick={() => toggleWadExpansion(wad.id, wad.type)}
              >
                <span class="type-icon">{getTypeIcon(wad.type)}</span>
                <span class="type-text">{getTypeDisplay(wad.type)}</span>
                <span class="wad-time">{formatTimestamp(wad.createdAt)}</span>
                <span class="spacer"></span>
                {#if wad.amounts.length === 1}
                  <span class="wad-amount"
                    >{formatAmount([wad.amounts[0]])}</span
                  >
                {/if}
                <span class="expand-icon"
                  >{isWadSelected(wad.id, wad.type) ? "▼" : "▶"}</span
                >
                <span
                  class="wad-status"
                  style="color: {getStatusColor(wad.status)}">{wad.status}</span
                >
              </button>
              {#if isWadSelected(wad.id, wad.type)}
                <div class="wad-details">
                  {#if wad.memo}
                    <div class="memo-section">
                      <div class="memo-header">{$t('labels.memo')}</div>
                      <div class="memo-text">{wad.memo}</div>
                    </div>
                  {/if}
                  <div class="balances-header">{$t('history.balancesHeader')}</div>
                  {#each wad.amounts as balance}
                    <div class="balance-item">
                      <span class="balance-amount"
                        >{formatAmount([balance])}</span
                      >
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      <div class="refresh-container">
        <button
          class="refresh-btn"
          onclick={loadWadHistory}
          disabled={loading || syncing}
        >
          {#if syncing}
            {$t('buttons.syncing')}
          {:else}
            {$t('buttons.refresh')}
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .history-page {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 70px; /* Account for navigation bar */
    display: flex;
    flex-direction: column;
    background: #ffffff;
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  .history-container {
    height: 100%;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .header {
    padding: 12px 0;
    border-bottom: 1px solid #eee;
    background: white;
    flex-shrink: 0;
  }

  .header h1 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    text-align: center;
  }

  .content {
    flex: 1;
    padding-bottom: 120px;
  }

  .history-list {
    padding: 0;
    margin: 0;
  }

  .wad-item {
    padding: 12px 16px;
    border-bottom: 1px solid #eee;
    width: 100%;
    box-sizing: border-box;
  }

  .wad-line {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
  }

  .wad-line.clickable {
    cursor: pointer;
    transition: background-color 0.2s ease;
    /* Reset button styles */
    border: none;
    background: none;
    font: inherit;
    text-align: left;
    width: 100%;
  }

  .wad-line.clickable:hover {
    background-color: #f8f9fa;
  }

  .wad-line.clickable:active {
    background-color: #e9ecef;
  }

  .type-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .type-text {
    font-size: 14px;
    font-weight: 600;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .wad-time {
    font-size: 12px;
    color: #666;
    flex-shrink: 0;
  }

  .spacer {
    flex: 1;
  }

  .wad-amount {
    font-size: 12px;
    font-weight: 500;
    color: #333;
    flex-shrink: 0;
    margin-right: 8px;
  }

  .expand-icon {
    font-size: 12px;
    color: #666;
    flex-shrink: 0;
    margin-right: 8px;
    transition: transform 0.2s ease;
  }

  .wad-status {
    font-size: 12px;
    font-weight: 500;
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .wad-details {
    background-color: #f8f9fa;
    border-radius: 8px;
    margin-top: 8px;
    padding: 12px;
    border: 1px solid #e9ecef;
  }

  .memo-section {
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid #dee2e6;
  }

  .memo-header {
    font-size: 12px;
    font-weight: 600;
    color: #666;
    margin-bottom: 4px;
    text-transform: uppercase;
  }

  .memo-text {
    font-size: 14px;
    color: #333;
    line-height: 1.4;
    word-wrap: break-word;
  }

  .balances-header {
    font-size: 12px;
    font-weight: 600;
    color: #666;
    margin-bottom: 8px;
    text-transform: uppercase;
  }

  .balance-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 0;
  }

  .balance-item:not(:last-child) {
    border-bottom: 1px solid #dee2e6;
  }

  .balance-amount {
    font-size: 14px;
    font-weight: 500;
    color: #333;
  }

  .refresh-container {
    position: fixed;
    bottom: 80px;
    left: 0;
    right: 0;
    padding: 12px;
    background: transparent;
    display: flex;
    justify-content: center;
    flex-shrink: 0;
    z-index: 100;
  }

  .refresh-btn {
    background: #007bff;
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 25px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(0, 123, 255, 0.3);
    transition: all 0.2s ease;
    min-width: 120px;
  }

  .refresh-btn:hover {
    background: #0056b3;
    box-shadow: 0 6px 16px rgba(0, 123, 255, 0.4);
    transform: translateY(-1px);
  }

  .refresh-btn:active {
    transform: translateY(0);
    box-shadow: 0 2px 8px rgba(0, 123, 255, 0.3);
  }

  .refresh-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .loading,
  .error,
  .empty {
    padding: 20px;
    text-align: center;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
  }

  .spinner {
    border: 2px solid #f3f3f3;
    border-top: 2px solid #007bff;
    border-radius: 50%;
    width: 20px;
    height: 20px;
    animation: spin 1s linear infinite;
    margin: 0 auto 10px;
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }
</style>
