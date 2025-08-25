<script lang="ts">
  import { getCurrencies, setPriceProviderCurrency } from "../../commands";
  import { displayCurrency } from "../../stores";

  interface Props {
    onClose?: () => void;
  }

  let { onClose }: Props = $props();

  let fiatCurrencies = $state<string[]>(["usd"]);

  getCurrencies().then((resp) => {
    if (resp) fiatCurrencies = resp;
  });
</script>

<div class="settings-container">
  <div class="modal-header">
    <h3>Settings</h3>
  </div>
  <div class="select-currency">
    <h3>Select your currency:</h3>
    <select
      name="deposit-token"
      value={$displayCurrency}
      onchange={(e) => {
        displayCurrency.set((e.target as HTMLSelectElement).value);
        setPriceProviderCurrency($displayCurrency);
      }}
      required
    >
      {#each fiatCurrencies as currency}
        <option value={currency}>
          {currency.toUpperCase()}
        </option>
      {/each}
    </select>
  </div>
  <button class="done-button" onclick={onClose}>Done</button>
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

  .done-button {
    background-color: #1e88e5;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 0.5rem 1rem;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .done-button:hover {
    background-color: #1976d2;
  }

  .done-button:active {
    background-color: #1565c0;
  }
</style>
