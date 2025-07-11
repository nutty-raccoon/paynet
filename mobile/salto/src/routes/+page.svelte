<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import PayButton from "./components/PayButton.svelte";
  import ReceiveButton from "./components/ReceiveButton.svelte";
  import PayModal from "./components/PayModal.svelte";
  import NavBar, { type Tab } from "./components/NavBar.svelte";
  import { type BalanceChange, type NodeData } from "../types";
  import NodesBalancePage from "./balances/NodesBalancePage.svelte";
  import {
    computeTotalBalancePerUnit,
    decreaseNodeBalance,
    formatBalance,
    increaseNodeBalance,
  } from "../utils";
  import { onMount, onDestroy } from "svelte";
  import { getNodesBalance } from "../commands";
  import { platform } from "@tauri-apps/plugin-os";
  import ScanModal from "./scan/ScanModal.svelte";

  const currentPlatform = platform();
  const isMobile = currentPlatform == "ios" || currentPlatform == "android";

  // Sample data with multiple nodes to demonstrate the new card design
  let nodes: NodeData[] = $state([]);

  let activeTab: Tab = $state("pay");
  let isPayModalOpen = $state(false);
  let isReceiveModalOpen = $state(false);
  let errorMessage = $state("");

  // Calculate total balance across all nodes
  let totalBalance: Map<string, number> = $derived(
    computeTotalBalancePerUnit(nodes),
  );
  let formattedTotalBalance: string[] = $derived(
    totalBalance
      .entries()
      .map(([unit, amount]) => {
        const formatted = formatBalance({ unit, amount });
        return `${formatted.asset}: ${formatted.amount}`;
      })
      .toArray(),
  );

  // Effect to manage scrolling based on active tab
  $effect(() => {
    document.body.classList.add("no-scroll");
  });

  const onAddNode = (nodeData: NodeData) => {
    nodes.push(nodeData);
  };

  const onNodeBalanceIncrease = (balanceIncrease: BalanceChange) => {
    increaseNodeBalance(nodes, balanceIncrease);
  };
  const onNodeBalanceDecrease = (balanceIncrease: BalanceChange) => {
    decreaseNodeBalance(nodes, balanceIncrease);
  };

  const onReceiveError = (error: string) => {
    errorMessage = error;
  };

  // PayModal control functions
  function openPayModal() {
    isPayModalOpen = true;
    // Add history entry to handle back button
    history.pushState({ modal: true }, "");
  }

  function closePayModal() {
    isPayModalOpen = false;
  }

  function openReceiveModal() {
    isReceiveModalOpen = true;
    // Add history entry to handle back button
    history.pushState({ modal: true }, "");
  }

  function closeReceiveModal() {
    isReceiveModalOpen = false;
  }

  // Set up back button listener for PayModal
  function handlePopState() {
    if (isPayModalOpen) {
      closePayModal();
    } else if (isReceiveModalOpen) {
      closeReceiveModal();
    }
  }

  onMount(() => {
    getNodesBalance().then((nodesData) => {
      if (!!nodesData) {
        nodesData.forEach(onAddNode);
      }
    });

    listen<BalanceChange>("balance-increase", (event) => {
      onNodeBalanceIncrease(event.payload);
    });
    listen<BalanceChange>("balance-decrease", (event) => {
      onNodeBalanceDecrease(event.payload);
    });
    // Add popstate listener for back button handling
    window.addEventListener("popstate", handlePopState);
  });

  // Clean up when component is destroyed
  onDestroy(() => {
    document.body.classList.remove("no-scroll");
    window.removeEventListener("popstate", handlePopState);
  });

  // Clean up when component is destroyed
  onDestroy(() => {
    document.body.classList.remove("no-scroll");
  });
</script>

<main class="container">
  {#if activeTab === "pay"}
    <div class="pay-container">
      <div class="total-balance-card">
        <h2 class="balance-title">TOTAL BALANCE</h2>
        <p class="total-balance-amount">{formattedTotalBalance}</p>
      </div>
      {#if errorMessage}
        <div class="error-message">
          {errorMessage}
        </div>
      {/if}
      <PayButton onClick={openPayModal} />
      <ReceiveButton
        {isMobile}
        onClick={openReceiveModal}
        onError={onReceiveError}
      />
    </div>
  {:else if activeTab === "balances"}
    <div class="balances-container">
      <NodesBalancePage {nodes} {onAddNode} />
    </div>
  {/if}
</main>

<NavBar
  {activeTab}
  onTabChange={(tab: Tab) => {
    activeTab = tab;
  }}
/>

<PayModal
  isOpen={isPayModalOpen}
  availableBalances={totalBalance}
  onClose={closePayModal}
/>

<ScanModal
  isOpen={isReceiveModalOpen}
  onCancell={closeReceiveModal}
  onSuccess={closeReceiveModal}
/>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;
    color: #0f0f0f;
    background-color: #ffffff;
    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    padding: 0;
  }

  /* Global style to disable scrolling - will be applied to body when needed */
  :global(body.no-scroll) {
    overflow: hidden;
    height: 100%;
    position: fixed;
    width: 100%;
  }

  .container {
    margin: 0;
    padding-top: 2rem;
    padding-bottom: 70px; /* Add space for the navigation bar */
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
    background-color: #ffffff;
    min-height: 100vh;
  }

  .pay-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    gap: 1.5rem;
    margin-top: 2rem;
  }

  .balances-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    margin-top: 1rem;
  }

  .total-balance-card {
    background-color: white;
    border-radius: 16px;
    padding: 1.5rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    width: 80%;
    max-width: 400px;
    text-align: center;
  }

  .balance-title {
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #666;
    margin: 0 0 0.5rem 0;
    font-weight: 600;
  }

  .total-balance-amount {
    font-size: 2.5rem;
    font-weight: 700;
    color: #0f0f0f;
    margin: 0;
  }

  .error-message {
    background-color: #fee2e2;
    color: #dc2626;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    font-size: 0.875rem;
    font-weight: 500;
    text-align: center;
    border: 1px solid #fecaca;
    width: 90%;
    max-width: 400px;
    margin: 0 auto;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #0f0f0f;
      background-color: #ffffff;
    }
  }
</style>
