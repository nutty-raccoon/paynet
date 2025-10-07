<script lang="ts">
  import {
    getCurrencies,
    setPriceProviderCurrency,
    getSeedPhrase,
  } from "../../commands";
  import { displayCurrency, showErrorDetail, currentLanguage } from "../../stores";
  import { t } from "../../stores/i18n";
  import SeedPhraseCard from "../components/SeedPhraseCard.svelte";

  interface Props {
    onClose?: () => void;
  }

  let { onClose }: Props = $props();

  let fiatCurrencies = $state<string[]>(["usd"]);
  let seedPhrase = $state<string>("");
  let showSeedPhrase = $state<boolean>(false);
  let isLoadingSeed = $state<boolean>(false);

  getCurrencies().then((resp) => {
    if (resp) fiatCurrencies = resp;
  });

  const handleShowSeedPhrase = async () => {
    if (!!showSeedPhrase) {
      // Hide the seed phrase
      showSeedPhrase = false;
      seedPhrase = ""; // Clear the seed phrase from memory
    } else {
      // Show the seed phrase - always fetch fresh from command
      isLoadingSeed = true;
      const phrase = await getSeedPhrase();
      if (!!phrase) {
        seedPhrase = phrase;
        showSeedPhrase = true;
      }
      isLoadingSeed = false;
    }
  };
</script>

<div class="settings-container">
  <div class="modal-header">
    <h3>{$t('settings.title')}</h3>
  </div>
  <div class="select-currency">
    <h3>{$t('settings.currency.title')}</h3>
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

  <div class="select-language">
    <h3>{$t('settings.language.title')}</h3>
    <select
      name="language"
      value={$currentLanguage}
      onchange={(e) => {
        currentLanguage.set((e.target as HTMLSelectElement).value);
      }}
      required
    >
      <option value="en">{$t('settings.language.english')}</option>
      <option value="es">{$t('settings.language.spanish')}</option>
    </select>
  </div>

  <div class="toggle-section">
    <h3>{$t('settings.errorDetails.title')}</h3>
    <button
      class="toggle-button {$showErrorDetail ? 'active' : ''}"
      onclick={() => showErrorDetail.set(!$showErrorDetail)}
    >
      {$showErrorDetail ? $t('settings.errorDetails.hide') : $t('settings.errorDetails.show')}
    </button>
  </div>

  <div class="seed-phrase-section">
    <h3>{$t('settings.walletRecovery.title')}</h3>
    <button
      class="show-seed-button"
      onclick={handleShowSeedPhrase}
      disabled={isLoadingSeed}
    >
      {isLoadingSeed
        ? $t('settings.walletRecovery.loadingSeed')
        : showSeedPhrase
          ? $t('settings.walletRecovery.hideSeedPhrase')
          : $t('settings.walletRecovery.showSeedPhrase')}
    </button>

    {#if showSeedPhrase && seedPhrase}
      <div class="seed-phrase-container">
        <p class="warning-text">
          {$t('settings.walletRecovery.warning')}
        </p>
        <SeedPhraseCard {seedPhrase} />
      </div>
    {/if}
  </div>

  <button class="done-button" onclick={onClose}>{$t('common.done')}</button>
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

  .select-currency,
  .select-language {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    width: 100%;
  }

  .select-currency select,
  .select-language select {
    margin-left: 1rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    background-color: white;
    cursor: pointer;
    padding: 0.5rem;
  }

  .select-currency select:focus,
  .select-language select:focus {
    border-color: #1e88e5;
    outline: none;
    box-shadow: 0 0 0 2px rgba(30, 136, 229, 0.2);
  }

  .toggle-section {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    width: 100%;
  }

  .toggle-section h3 {
    margin: 0;
    font-size: 1.2rem;
    color: #333;
  }

  .toggle-button {
    background-color: #757575;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 0.5rem 1rem;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s;
    margin-left: 1rem;
  }

  .toggle-button:hover {
    background-color: #616161;
  }

  .toggle-button.active {
    background-color: #4caf50;
  }

  .toggle-button.active:hover {
    background-color: #45a049;
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

  .seed-phrase-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    width: 100%;
    margin: 1.5rem 0;
  }

  .seed-phrase-section h3 {
    margin: 0;
    font-size: 1.2rem;
    color: #333;
  }

  .show-seed-button {
    background-color: #ff9800;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 0.5rem 1rem;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .show-seed-button:hover:not(:disabled) {
    background-color: #f57c00;
  }

  .show-seed-button:active:not(:disabled) {
    background-color: #ef6c00;
  }

  .show-seed-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }

  .seed-phrase-container {
    width: 100%;
    max-width: 400px;
  }

  .warning-text {
    font-size: 0.9rem;
    color: #dc2626;
    margin: 0 0 1rem 0;
    line-height: 1.5;
    font-weight: 500;
    text-align: center;
  }
</style>
