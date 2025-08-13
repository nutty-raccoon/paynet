<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import SendModal from "./send/SendModal.svelte";
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
  import { getNodesBalance, checkWalletExists } from "../commands";
  import ReceiveModal from "./receive/ReceiveModal.svelte";
  import InitPage from "./init/InitPage.svelte";
  import WadHistoryPage from "./components/WadHistoryPage.svelte";

  const Modal = {
    ROOT: 0,
    SEND: 1,
    RECEIVE: 2,
  } as const;
  type Modal = (typeof Modal)[keyof typeof Modal];

  let currentModal = $state<Modal>(Modal.ROOT);

  // Sample data with multiple nodes to demonstrate the new card design
  let nodes: NodeData[] = $state([]);

  let activeTab: Tab = $state("pay");
  let errorMessage = $state("");
  let walletExists = $state<boolean | null>(null); // null = loading, true/false = result

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
    // Allow scrolling for history page
    if (activeTab === "history") {
      document.body.classList.remove("no-scroll");
    } else {
      document.body.classList.add("no-scroll");
    }
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

  const onWalletInitialized = (initialTab: Tab = "pay") => {
    walletExists = true;
    activeTab = initialTab;
  };

  // SendModal control functions
  function openModal(modal: Modal) {
    currentModal = modal;
    // Add history entry to handle back button
    history.pushState({ modal: true }, "");
  }

  function goBackToRoot() {
    currentModal = Modal.ROOT;
  }

  // Set up back button listener for SendModal
  function handlePopState() {
    goBackToRoot();
  }

  onMount(async () => {
    // First check if wallet exists
    const exists = await checkWalletExists();
    walletExists = exists;

    if (exists) {
      // Only load wallet data if wallet exists
      getNodesBalance().then((nodesData) => {
        if (!!nodesData) {
          nodesData.forEach(onAddNode);
        }
      });
    }

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
  {#if walletExists === null}
    <!-- Loading state -->
    <div class="loading-container">
      <p>Loading...</p>
    </div>
  {:else if walletExists === false}
    <!-- Show initialization page -->
    <InitPage {onWalletInitialized} />
  {:else}
    <!-- Show main app content -->
    {#if activeTab === "pay"}
      {#if currentModal == Modal.ROOT}
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
          <button class="pay-button" onclick={() => openModal(Modal.SEND)}
            >Send</button
          >
          <button
            class="receive-button"
            onclick={() => openModal(Modal.RECEIVE)}>Receive</button
          >
        </div>
      {:else if currentModal == Modal.SEND}
        <SendModal availableBalances={totalBalance} onClose={goBackToRoot} />
      {:else if currentModal == Modal.RECEIVE}
        <ReceiveModal onClose={goBackToRoot} />
      {/if}
    {:else if activeTab === "balances"}
      <div class="balances-container">
        <NodesBalancePage {nodes} {onAddNode} />
      </div>
    {:else if activeTab === "history"}
      <WadHistoryPage />
    {/if}
  {/if}
</main>

{#if walletExists}
  <NavBar
    {activeTab}
    onTabChange={(tab: Tab) => {
      activeTab = tab;
    }}
  />
{/if}

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

  .pay-button {
    background-color: #1e88e5;
    color: white;
    font-size: 1.2rem;
    font-weight: 600;
    padding: 0.8rem 3rem;
    border: none;
    border-radius: 50px;
    cursor: pointer;
    transition:
      background-color 0.2s,
      transform 0.1s;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  }

  .pay-button:hover {
    background-color: #1976d2;
  }

  .pay-button:active {
    transform: scale(0.98);
    background-color: #1565c0;
  }

  .receive-button {
    background-color: #2e7d32;
    color: white;
    font-size: 1.2rem;
    font-weight: 600;
    padding: 0.8rem 3rem;
    border: none;
    border-radius: 50px;
    cursor: pointer;
    transition:
      background-color 0.2s,
      transform 0.1s;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  }

  .receive-button:hover {
    background-color: #1b5e20;
  }

  .receive-button:active {
    transform: scale(0.98);
    background-color: #0d4814;
  }

  .loading-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 50vh;
    font-size: 1.2rem;
    color: #666;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #0f0f0f;
      background-color: #ffffff;
    }
  }
</style>
