<script lang="ts">
  import { initWallet, restoreWallet } from "../../commands";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
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
  let errorMessage = $state("");
  let isLoading = $state(false);
  let hasSavedSeedPhrase = $state(false);

  const handleCreateNew = async () => {
    isLoading = true;
    errorMessage = "";

    try {
      const response = await initWallet();
      if (response) {
        seedPhrase = response.seedPhrase;
        currentMode = InitMode.SHOW_SEED;
      } else {
        errorMessage = "Failed to create wallet";
      }
    } catch (error) {
      errorMessage = `Failed to create wallet: ${error}`;
    } finally {
      isLoading = false;
    }
  };

  const handleRestore = async () => {
    if (!restoreSeedPhrase.trim()) {
      errorMessage = "Please enter your seed phrase";
      return;
    }

    isLoading = true;
    errorMessage = "";

    try {
      await restoreWallet(restoreSeedPhrase.trim());
      currentMode = InitMode.RECOVERY_SUCCESS;
    } catch (error) {
      errorMessage = `Failed to restore wallet: ${error}`;
    } finally {
      isLoading = false;
    }
  };

  const handleFinishSetup = () => {
    if (!hasSavedSeedPhrase) {
      errorMessage = "Please confirm you have saved your seed phrase";
      return;
    }
    onWalletInitialized("pay");
  };

  const handleRecoveryNext = () => {
    onWalletInitialized("balances");
  };

  const goBack = () => {
    errorMessage = "";
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
      <h1 class="title">Welcome to Salto Wallet</h1>
      <p class="subtitle">
        Get started by creating a new wallet or restoring an existing one
      </p>

      <div class="button-group">
        <button
          class="primary-button"
          onclick={() => (currentMode = InitMode.CREATE_NEW)}
          disabled={isLoading}
        >
          Create New Wallet
        </button>

        <button
          class="secondary-button"
          onclick={() => (currentMode = InitMode.RESTORE)}
          disabled={isLoading}
        >
          Restore Existing Wallet
        </button>
      </div>
    </div>
  {:else if currentMode === InitMode.CREATE_NEW}
    <div class="create-container">
      <h2 class="section-title">Create New Wallet</h2>
      <p class="description">
        A new wallet will be created with a unique seed phrase that you can use
        to recover your wallet.
      </p>

      <div class="button-group">
        <button
          class="primary-button"
          onclick={handleCreateNew}
          disabled={isLoading}
        >
          {isLoading ? "Creating..." : "Create Wallet"}
        </button>

        <button class="secondary-button" onclick={goBack} disabled={isLoading}>
          Back
        </button>
      </div>
    </div>
  {:else if currentMode === InitMode.SHOW_SEED}
    <div class="seed-container">
      <h2 class="section-title">Your Seed Phrase</h2>
      <p class="warning-text">
        Your wallet has been created. Write down this seed phrase and store it
        in a safe place. You'll need it to recover your wallet.
      </p>

      <SeedPhraseCard {seedPhrase} />

      <div class="checkbox-container">
        <label class="checkbox-label">
          <input
            type="checkbox"
            bind:checked={hasSavedSeedPhrase}
            class="checkbox"
          />
          I have safely stored my seed phrase
        </label>
      </div>

      <div class="button-group">
        <button
          class="primary-button"
          onclick={handleFinishSetup}
          disabled={!hasSavedSeedPhrase}
        >
          Continue
        </button>
      </div>
    </div>
  {:else if currentMode === InitMode.RESTORE}
    <div class="restore-container">
      <h2 class="section-title">Restore Wallet</h2>
      <p class="description">
        Enter your seed phrase to restore your existing wallet.
      </p>

      <div class="input-group">
        <label for="seedPhrase" class="input-label">Seed Phrase</label>
        <textarea
          id="seedPhrase"
          bind:value={restoreSeedPhrase}
          placeholder="Enter your seed phrase here..."
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
          {isLoading ? "Restoring..." : "Restore Wallet"}
        </button>

        <button class="secondary-button" onclick={goBack} disabled={isLoading}>
          Back
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

      <h2 class="success-title">Recovery Successful!</h2>
      <p class="success-description">
        Your wallet has been successfully restored. Now add back the nodes you
        used to deposit funds, and we will get your money back.
      </p>

      <div class="button-group">
        <button class="primary-button" onclick={handleRecoveryNext}>
          Next
        </button>
      </div>
    </div>
  {/if}

  {#if errorMessage}
    <div class="error-message">
      {errorMessage}
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
