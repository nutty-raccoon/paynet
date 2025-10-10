<script lang="ts">
  import { initWallet, restoreWallet } from "../../commands";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { showErrorToast } from "../../stores/toast";
  import { t } from "../../stores/i18n";
  import SeedPhraseCard from "../components/SeedPhraseCard.svelte";

  interface Props {
    onWalletInitialized: (initialTab?: "pay" | "balances") => void;
  }

  let { onWalletInitialized }: Props = $props();

  const InitMode = {
    CHOICE: 0,
    CREATE_NEW: 1,
    RESTORE: 2,
    SHOW_SEED: 3,
    RECOVERY_SUCCESS: 4,
  } as const;
  type InitMode = (typeof InitMode)[keyof typeof InitMode];

  let currentMode = $state<InitMode>(InitMode.CHOICE);
  let seedPhrase = $state("");
  let restoreSeedPhrase = $state("");
  let validationError = $state("");
  let isLoading = $state(false);
  let hasSavedSeedPhrase = $state(false);

  const handleCreateNew = async () => {
    isLoading = true;
    validationError = "";

    try {
      const response = await initWallet();
      if (response) {
        seedPhrase = response.seedPhrase;
        currentMode = InitMode.SHOW_SEED;
      }
      // Error handling is now done in the command function via toast
    } catch (error) {
      // Critical errors are handled by the command function via toast
      console.error("Unexpected error in handleCreateNew:", error);
    } finally {
      isLoading = false;
    }
  };

  const handleRestore = async () => {
    if (!restoreSeedPhrase.trim()) {
      validationError = $t('wallet.enterSeedPhrase');
      return;
    }

    isLoading = true;
    validationError = "";

    try {
      const result = await restoreWallet(restoreSeedPhrase.trim());
      if (result !== undefined) {
        currentMode = InitMode.RECOVERY_SUCCESS;
      }
      // Error handling is now done in the command function via toast
    } catch (error) {
      // Critical errors are handled by the command function via toast
      console.error("Unexpected error in handleRestore:", error);
    } finally {
      isLoading = false;
    }
  };

  const handleFinishSetup = () => {
    if (!hasSavedSeedPhrase) {
      validationError = $t('wallet.confirmSaved');
      return;
    }
    onWalletInitialized("pay");
  };

  const handleRecoveryNext = () => {
    onWalletInitialized("balances");
  };

  const goBack = () => {
    validationError = "";
    if (
      currentMode === InitMode.CREATE_NEW ||
      currentMode === InitMode.RESTORE
    ) {
      currentMode = InitMode.CHOICE;
      restoreSeedPhrase = "";
    }
  };
</script>

<div class="init-container">
  {#if currentMode === InitMode.CHOICE}
    <div class="choice-container">
      <h1 class="title">{$t('wallet.welcome')}</h1>
      <p class="subtitle">
        {$t('wallet.getStarted')}
      </p>

      <div class="button-group">
        <button
          class="primary-button"
          onclick={() => (currentMode = InitMode.CREATE_NEW)}
          disabled={isLoading}
        >
          {$t('wallet.createNew')}
        </button>

        <button
          class="secondary-button"
          onclick={() => (currentMode = InitMode.RESTORE)}
          disabled={isLoading}
        >
          {$t('wallet.restoreExisting')}
        </button>
      </div>
    </div>
  {:else if currentMode === InitMode.CREATE_NEW}
    <div class="create-container">
      <h2 class="section-title">{$t('wallet.createNew')}</h2>
      <p class="description">
        {$t('wallet.newWalletDesc')}
      </p>

      <div class="button-group">
        <button
          class="primary-button"
          onclick={handleCreateNew}
          disabled={isLoading}
        >
          {isLoading ? $t('wallet.creating') : $t('wallet.create')}
        </button>

        <button class="secondary-button" onclick={goBack} disabled={isLoading}>
          {$t('common.back')}
        </button>
      </div>
    </div>
  {:else if currentMode === InitMode.SHOW_SEED}
    <div class="seed-container">
      <h2 class="section-title">{$t('wallet.yourSeedPhrase')}</h2>
      <p class="warning-text">
        {$t('wallet.walletCreatedDesc')}
      </p>

      <SeedPhraseCard {seedPhrase} />

      <div class="checkbox-container">
        <label class="checkbox-label">
          <input
            type="checkbox"
            bind:checked={hasSavedSeedPhrase}
            class="checkbox"
          />
          {$t('wallet.savedSeedPhrase')}
        </label>
      </div>

      <div class="button-group">
        <button
          class="primary-button"
          onclick={handleFinishSetup}
          disabled={!hasSavedSeedPhrase}
        >
          {$t('wallet.continue')}
        </button>
      </div>
    </div>
  {:else if currentMode === InitMode.RESTORE}
    <div class="restore-container">
      <h2 class="section-title">{$t('wallet.restoreWallet')}</h2>
      <p class="description">
        {$t('wallet.restoreWalletDesc')}
      </p>

      <div class="input-group">
        <label for="seedPhrase" class="input-label">{$t('wallet.seedPhraseLabel')}</label>
        <textarea
          id="seedPhrase"
          bind:value={restoreSeedPhrase}
          placeholder={$t('placeholders.seedPhrase')}
          class="seed-input"
          rows="4"
          disabled={isLoading}
        ></textarea>
      </div>

      <div class="button-group">
        <button
          class="primary-button"
          onclick={handleRestore}
          disabled={isLoading || !restoreSeedPhrase.trim()}
        >
          {isLoading ? $t('wallet.restoring') : $t('wallet.restore')}
        </button>

        <button class="secondary-button" onclick={goBack} disabled={isLoading}>
          {$t('common.back')}
        </button>
      </div>
    </div>
  {:else if currentMode === InitMode.RECOVERY_SUCCESS}
    <div class="success-container">
      <div class="success-icon">
        <svg
          width="64"
          height="64"
          viewBox="0 0 24 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <circle cx="12" cy="12" r="10" fill="#10b981" />
          <path
            d="M9 12l2 2 4-4"
            stroke="white"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </div>

      <h2 class="success-title">{$t('wallet.recoverySuccessful')}</h2>
      <p class="success-description">
        {$t('wallet.walletRestoredDesc')}
      </p>

      <div class="button-group">
        <button class="primary-button" onclick={handleRecoveryNext}>
          {$t('wallet.next')}
        </button>
      </div>
    </div>
  {/if}

  {#if validationError}
    <div class="error-message">
      {validationError}
    </div>
  {/if}
</div>

<style>
  .init-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 2rem;
    background-color: #ffffff;
  }

  .choice-container,
  .create-container,
  .seed-container,
  .restore-container,
  .success-container {
    width: 100%;
    max-width: 500px;
    text-align: center;
  }

  .title {
    font-size: 2.5rem;
    font-weight: 700;
    color: #0f0f0f;
    margin: 0 0 1rem 0;
  }

  .section-title {
    font-size: 2rem;
    font-weight: 600;
    color: #0f0f0f;
    margin: 0 0 1rem 0;
  }

  .subtitle,
  .description {
    font-size: 1.1rem;
    color: #666;
    margin: 0 0 2rem 0;
    line-height: 1.5;
  }

  .warning-text {
    font-size: 1rem;
    color: #dc2626;
    margin: 0 0 1.5rem 0;
    line-height: 1.5;
    font-weight: 500;
  }

  .button-group {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    margin-top: 2rem;
  }

  .primary-button {
    background-color: #1e88e5;
    color: white;
    font-size: 1.2rem;
    font-weight: 600;
    padding: 1rem 2rem;
    border: none;
    border-radius: 12px;
    cursor: pointer;
    transition:
      background-color 0.2s,
      transform 0.1s;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  }

  .primary-button:hover:not(:disabled) {
    background-color: #1976d2;
  }

  .primary-button:active:not(:disabled) {
    transform: scale(0.98);
    background-color: #1565c0;
  }

  .primary-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }

  .secondary-button {
    background-color: transparent;
    color: #1e88e5;
    font-size: 1.1rem;
    font-weight: 500;
    padding: 0.8rem 2rem;
    border: 2px solid #1e88e5;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .secondary-button:hover:not(:disabled) {
    background-color: #1e88e5;
    color: white;
  }

  .secondary-button:disabled {
    border-color: #ccc;
    color: #ccc;
    cursor: not-allowed;
  }

  .seed-phrase-box {
    background-color: #f8f9fa;
    border: 2px solid #e9ecef;
    border-radius: 12px;
    padding: 1.5rem;
    margin: 1.5rem 0;
  }

  .seed-phrase-text {
    font-family: "Courier New", monospace;
    font-size: 1rem;
    color: #0f0f0f;
    line-height: 1.6;
    margin: 0;
    word-break: break-word;
  }

  .checkbox-container {
    margin: 1.5rem 0;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    font-size: 1rem;
    color: #0f0f0f;
    cursor: pointer;
  }

  .checkbox {
    width: 1.2rem;
    height: 1.2rem;
    cursor: pointer;
  }

  .input-group {
    margin: 1.5rem 0;
    text-align: left;
  }

  .input-label {
    display: block;
    font-size: 1rem;
    font-weight: 500;
    color: #0f0f0f;
    margin-bottom: 0.5rem;
  }

  .seed-input {
    width: 100%;
    padding: 1rem;
    border: 2px solid #e9ecef;
    border-radius: 8px;
    font-size: 1rem;
    font-family: inherit;
    resize: vertical;
    min-height: 100px;
  }

  .seed-input:focus {
    outline: none;
    border-color: #1e88e5;
  }

  .seed-input:disabled {
    background-color: #f8f9fa;
    cursor: not-allowed;
  }

  .success-icon {
    margin: 0 auto 1.5rem;
    display: flex;
    justify-content: center;
  }

  .success-title {
    font-size: 2rem;
    font-weight: 600;
    color: #10b981;
    margin: 0 0 1rem 0;
  }

  .success-description {
    font-size: 1.1rem;
    color: #666;
    margin: 0 0 2rem 0;
    line-height: 1.6;
  }

  .error-message {
    background-color: #fee2e2;
    color: #dc2626;
    padding: 1rem;
    border-radius: 8px;
    font-size: 0.9rem;
    font-weight: 500;
    text-align: center;
    border: 1px solid #fecaca;
    margin-top: 1rem;
    max-width: 500px;
  }

</style>
