<script lang="ts">
  import { emit } from "@tauri-apps/api/event";
  import { pushState } from "$app/navigation";
  import SendModal from "./send/SendModal.svelte";
  import NavBar, { type Tab } from "./components/NavBar.svelte";
  import NodesBalancePage from "./balances/NodesBalancePage.svelte";
  import { formatBalance, getTotalAmountInDisplayCurrency } from "../utils";
  import { onMount, onDestroy } from "svelte";
  import { checkWalletExists, getPendingQuotes } from "../commands";
  import ReceiveModal from "./receive/ReceiveModal.svelte";
  import SettingsModal from "./settings/SettingsPage.svelte";
  import InitPage from "./init/InitPage.svelte";
  import { displayCurrency, tokenPrices, totalBalance } from "../stores";
  import WadHistoryPage from "./components/WadHistoryPage.svelte";
  import Toast from "../components/Toast.svelte";
  import { page } from "$app/state";

  const Modal = {
    ROOT: 0,
    SEND: 1,
    RECEIVE: 2,
    SETTINGS: 3,
  } as const;
  type Modal = (typeof Modal)[keyof typeof Modal];

  let currentModal = $state<Modal>(Modal.ROOT);

  let activeTab: Tab = $state("pay");
  let errorMessage = $state("");
  let walletExists = $state<boolean | null>(null); // null = loading, true/false = result
  let formattedBalance: string[] = $derived(
    $totalBalance
      .entries()
      .map(([unit, amount]) => {
        const { asset, assetAmount } = formatBalance(unit, amount);
        return `${assetAmount} ${asset}`;
      })
      .toArray(),
  );
  let totalAmount = $derived(
    !!$tokenPrices
      ? getTotalAmountInDisplayCurrency($totalBalance, $tokenPrices)
      : null,
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

  const onWalletInitialized = (initialTab: Tab = "pay") => {
    walletExists = true;
    activeTab = initialTab;
  };

  const onCloseSettings = () => {
    // Use history.back() to trigger the popstate handler
    history.back();
  };

  // SendModal control functions
  function openModal(modal: Modal) {
    currentModal = modal;
    // Add history entry to handle back button
    pushState("", { modal: true });
  }

  function goBackToRoot() {
    currentModal = Modal.ROOT;
  }

  onMount(async () => {
    walletExists = await checkWalletExists();

    emit("front-ready", {});

    // Add popstate listener for back button handling
    window.addEventListener("popstate", handlePopState, { capture: true });

    if (walletExists) {
      // Only load pending quotes data manually on startup
      // Node balances are handled by the polling store
      getPendingQuotes();
    }
  });

  // Clean up when component is destroyed
  onDestroy(() => {
    document.body.classList.remove("no-scroll");
    window.removeEventListener("popstate", handlePopState, { capture: true });
  });

  // Clean up when component is destroyed
  onDestroy(() => {
    document.body.classList.remove("no-scroll");
  });

  function handlePopState(e: any) {
    const pageState = page.state as any;
    // At this point, page.state should still contain the data
    if (pageState?.previousTab) {
      activeTab = pageState.previousTab;
    }

    if (pageState.modal === true) {
      currentModal = Modal.ROOT;
    }
  }
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
    <!-- Settigs button float above all pages -->
    {#if activeTab !== "settings"}
      <button
        class="settings"
        onclick={() => {
          pushState("", { previousTab: activeTab });
          activeTab = "settings";
        }}>Settings</button
      >
    {/if}

    <!-- Show main app content -->
    {#if activeTab === "pay"}
      {#if currentModal == Modal.ROOT}
        <div class="pay-container">
          <div class="total-balance-card">
            <h2 class="balance-title">TOTAL BALANCE</h2>
            {#if !!totalAmount}
              <p class="total-currency-amount">
                {totalAmount.toFixed(2)}
                {$displayCurrency}
              </p>
            {:else}
              {#each formattedBalance as formatBalance}
                <p class="total-balance-amount">
                  {formatBalance}
                </p>
              {/each}
            {/if}
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
        <SendModal availableBalances={$totalBalance} onClose={goBackToRoot} />
      {:else if currentModal == Modal.RECEIVE}
        <ReceiveModal onClose={goBackToRoot} />
      {/if}
    {:else if activeTab === "balances"}
      <div class="balances-container">
        <NodesBalancePage />
      </div>
    {:else if activeTab === "settings"}
      <SettingsModal onClose={onCloseSettings} />
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

<!-- Toast notifications - rendered at app root level -->
<Toast />

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
    font-size: 2rem;
    font-weight: 1000;
    color: #0f0f0f;
    margin: 0;
  }

  .total-currency-amount {
    font-size: 1.5rem;
    font-weight: 500;
    color: #0f0f0f;
    margin-top: 1rem;
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

  .settings {
    position: fixed;
    top: 1rem;
    right: 1rem;
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
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

  .settings {
    position: fixed;
    top: 1rem;
    right: 1rem;
    background-color: #ffffff;
    border: 2px solid #e0e0e0;
    border-radius: 8px;
    padding: 0.5rem 1rem;
    font-size: 1rem;
    font-weight: 500;
    color: #0f0f0f;
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .settings:hover {
    border-color: #bdbdbd;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
    transform: translateY(-1px);
  }
</style>
