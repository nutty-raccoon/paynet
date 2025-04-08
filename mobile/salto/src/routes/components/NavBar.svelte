<script lang="ts">
  // Props: active tab and callback for when tab is changed
  export let activeTab: 'pay' | 'balances' = 'pay';
  export let onTabChange: (tab: 'pay' | 'balances') => void;
  
  function handleTabClick(tab: 'pay' | 'balances') {
    if (tab !== activeTab) {
      onTabChange(tab);
    }
  }
</script>

<nav class="navbar">
  <div
    class="tab-item {activeTab === 'pay' ? 'active' : ''}"
    on:click={() => handleTabClick('pay')}
  >
    <div class="icon">💸</div>
    <span>Pay</span>
  </div>
  
  <div
    class="tab-item {activeTab === 'balances' ? 'active' : ''}"
    on:click={() => handleTabClick('balances')}
  >
    <div class="icon">💰</div>
    <span>Balances</span>
  </div>
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