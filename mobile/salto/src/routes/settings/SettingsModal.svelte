<script lang="ts">
  import { fiatCurrenciesStored, selectedCurrencyStored } from "../../stores";
</script>

<div class="settings-container">
  <div class="modal-header">
    <h3>Settings</h3>
  </div>
  <div class="select-currency">
    <h3>Select your currency:</h3>
    <select
      name="deposit-token"
      value={$selectedCurrencyStored}
      on:change={(e) =>
        selectedCurrencyStored.set((e.target as HTMLSelectElement).value)}
      required
    >
      {#each $fiatCurrenciesStored as currency}
        <option value={currency}>
          {currency.toUpperCase()}
        </option>
      {/each}
    </select>
  </div>
</div>

<style>
  .settings-container {
    display: flex;
    flex-direction: column;
    width: 90%;
    max-width: 400px;
    gap: 1rem;
    margin: 0 auto;
    align-items: center;
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

  .select-currency {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
  }

  .select-currency select {
    margin-left: 1rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    background-color: white;
    cursor: pointer;
  }

  .select-currency select:focus {
    border-color: #1e88e5;
    outline: none;
    box-shadow: 0 0 0 2px rgba(30, 136, 229, 0.2);
  }
</style>
