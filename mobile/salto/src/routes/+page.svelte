<script lang="ts">
  import NodesBalancePages from "./components/NodesBalancePage.svelte";
  import PayButton from "./components/PayButton.svelte";
  import NavBar, { type Tab } from "./components/NavBar.svelte";
  import { type Node } from "../types";
  import NodesBalancePage from "./components/NodesBalancePage.svelte";
  import { formatBalance } from "../utils";
  import { onMount, onDestroy } from "svelte";
  import { get_nodes_balance } from "../commands";

  // Sample data with multiple nodes to demonstrate the new card design
  let nodes: Node[] = $state([
    { url: "http://localhost:20001", balance: 532 },
    { url: "https://node.example.com:8080", balance: 217.5 },
    { url: "https://payment-node.network.org", balance: 104.25 },
  ]);

  let activeTab: Tab = $state("pay");
  // Calculate total balance across all nodes
  let totalBalance = $derived(
    nodes.reduce((total, node) => total + node.balance, 0),
  );
  let formattedTotalBalance = $derived(formatBalance(totalBalance));

  // Effect to manage scrolling based on active tab
  $effect(() => {
    if (activeTab === "pay") {
      document.body.classList.add("no-scroll");
    } else {
      document.body.classList.remove("no-scroll");
    }
  });

  onMount(() => {
    get_nodes_balance();
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
      <PayButton />
    </div>
  {:else if activeTab === "balances"}
    <div class="balances-container">
      <NodesBalancePage {nodes} />
    </div>
  {/if}
</main>

<NavBar
  {activeTab}
  onTabChange={(tab: Tab) => {
    activeTab = tab;
  }}
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

  /* Global style to disable scrolling - will be applied to body when needed */
  :global(body.no-scroll) {
    overflow: hidden;
    height: 100%;
    position: fixed;
    width: 100%;
  }

  .container {
    margin: 0;
    padding-top: 10vh;
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

  @media (prefers-color-scheme: dark) {
    :root {
      color: #0f0f0f;
      background-color: #ffffff;
    }
  }
</style>
