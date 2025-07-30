<script lang="ts">
  export type Tab = "pay" | "balances" | "settings";

  interface Props {
    activeTab: Tab;
    onTabChange: (tab: Tab) => void;
  }

  let { activeTab, onTabChange }: Props = $props();

  function handleTabClick(tab: Tab) {
    if (tab !== activeTab) {
      onTabChange(tab);
    }
  }
</script>

<nav class="navbar">
  <button
    class="tab-item {activeTab === 'pay' ? 'active' : ''}"
    onclick={() => handleTabClick("pay")}
  >
    <div class="icon">💸</div>
    <span>Pay</span>
  </button>

  <button
    class="tab-item {activeTab === 'balances' ? 'active' : ''}"
    onclick={() => handleTabClick("balances")}
  >
    <div class="icon">💰</div>
    <span>Balances</span>
  </button>
</nav>

<style>
  .navbar {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    background-color: white;
    box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
    height: 70px;
  }

  .tab-item {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 8px 0;
    cursor: pointer;
    color: #888;
    transition: color 0.2s;
  }

  .tab-item.active {
    /* Text color */
    color: #1e88e5;
  }

  .icon {
    font-size: 1.5rem;
    margin-bottom: 4px;
  }

  span {
    font-size: 0.8rem;
    font-weight: 500;
  }
</style>
