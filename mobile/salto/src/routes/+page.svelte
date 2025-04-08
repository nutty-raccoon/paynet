<script lang="ts">
  import BalanceCard from './components/BalanceCard.svelte';
  import PayButton from './components/PayButton.svelte';
  import NavBar from './components/NavBar.svelte';
  import { onMount } from 'svelte';

  // State for active tab
  let activeTab: 'pay' | 'balances' = 'pay';
  
  // Handle tab changes from the navigation bar
  function handleTabChange(tab: 'pay' | 'balances') {
    activeTab = tab;
    console.log('Active tab changed to:', activeTab);
  }
</script>

<main class="container">
  {#if activeTab === 'pay'}
    <div class="pay-container">
      <BalanceCard balance={156} />
      <PayButton />
    </div>
  {:else if activeTab === 'balances'}
    <div class="balances-container">
      <h2>Your Balances</h2>
      <BalanceCard balance={156} />
      <BalanceCard balance={245.50} />
      <BalanceCard balance={78.25} />
    </div>
  {/if}
</main>

<NavBar activeTab={activeTab} onTabChange={handleTabChange} />

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
  
  .balances-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    gap: 1rem;
  }
  
  .balances-container h2 {
    color: #333;
    margin-bottom: 1.5rem;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #0f0f0f;
      background-color: #ffffff;
    }
  }
</style>