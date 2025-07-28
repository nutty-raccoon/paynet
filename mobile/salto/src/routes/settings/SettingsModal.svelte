<script lang="ts">
  export let selectedCurrency: string;
  export let fiatCurrencies: string[];
  export let onClose: () => void;

  const handleModalClose = () => {
    onClose();
  };
</script>

<div class="modal-overlay">
  <div class="modal-content">
    <div class="modal-header">
      <h3>Settings</h3>
      <button class="close-button" onclick={handleModalClose}>✕</button>
    </div>
    <div class="select-currency">
      <h3>Select your currency:</h3>
      <select name="deposit-token" bind:value={selectedCurrency} required>
        {#each fiatCurrencies as currency}
          <option value={currency}>
            {currency.toUpperCase()}
          </option>
        {/each}
      </select>
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
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }

  .modal-content {
    background: white;
    border-radius: 12px;
    width: 90%;
    max-width: 400px;
    padding: 1.5rem;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.5rem;
    color: #333;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    color: #666;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .close-button:hover {
    background-color: #f0f0f0;
  }

  .select-currency {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
  }

  .modal-content select {
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    background-color: white;
    cursor: pointer;
  }

  .modal-content select:focus {
    border-color: #1e88e5;
    outline: none;
    box-shadow: 0 0 0 2px rgba(30, 136, 229, 0.2);
  }
</style>
