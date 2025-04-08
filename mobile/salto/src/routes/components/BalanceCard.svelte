<script lang="ts">
  // Balance prop in USD
  export let balance: number = 0;
  
  // Format as USD
  $: formattedBalance = new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
  }).format(balance);
</script>

<div class="balance-card">
  <h2 class="balance-title">TOTAL BALANCE</h2>
  <p class="balance-amount">{formattedBalance}</p>
</div>

<style>
  .balance-card {
    background-color: white;
    border-radius: 16px;
    padding: 2rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    width: 80%;
    max-width: 400px;
    margin-bottom: 2rem;
  }

  .balance-title {
    font-size: 1rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #666;
    margin: 0 0 1rem 0;
    font-weight: 600;
  }

  .balance-amount {
    font-size: 3rem;
    font-weight: 700;
    color: #0f0f0f;
    margin: 0;
  }
</style>