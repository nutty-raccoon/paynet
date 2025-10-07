<script lang="ts">
  import type { EventHandler } from "svelte/elements";
  import { addNode } from "../../commands";
  import { showSuccessToast } from "../../stores/toast";
  import { t } from "../../stores/i18n";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let isLoading = $state(false);

  const handleFormSubmit: EventHandler<SubmitEvent, HTMLFormElement> = async (
    event,
  ) => {
    isLoading = true;
    event.preventDefault();
    const form = event.target as HTMLFormElement;
    const formDataObject = new FormData(form);
    const nodeAddress = formDataObject.get("node-address");

    if (!!nodeAddress) {
      let nodeAddressString = nodeAddress.toString();
      const result = await addNode(nodeAddressString);
      if (result !== undefined) {
        showSuccessToast($t('modals.nodeAddSuccess'));
        onClose();
      }
    }
    
    isLoading = false;
  };
</script>

<div class="modal-overlay">
  <div class="modal-content">
    <div class="modal-header">
      <h3>{$t('modals.addNode')}</h3>
      <button class="close-button" onclick={onClose}>✕</button>
    </div>

    <form onsubmit={handleFormSubmit}>
      <div class="form-group">
        <label for="node-address">{$t('forms.nodeAddress')}</label>
        <input
          type="url"
          id="node-address"
          name="node-address"
          placeholder="https://example.com"
          required
        />
      </div>

      <button type="submit" class="submit-button" disabled={isLoading}>
        {isLoading ? $t('modals.addingNode') : $t('modals.addNode')}
      </button>
    </form>
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

  .form-group {
    margin-bottom: 1.5rem;
  }

  .form-group label {
    display: block;
    font-size: 0.9rem;
    margin-bottom: 0.5rem;
    color: #333;
  }

  .form-group input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
    box-sizing: border-box;
  }

  .form-group input:focus {
    border-color: #1e88e5;
    outline: none;
    box-shadow: 0 0 0 2px rgba(30, 136, 229, 0.2);
  }

  .submit-button {
    padding: 0.8rem 2rem;
    background-color: #1e88e5;
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    width: 100%;
    transition: background-color 0.2s;
  }

  .submit-button:hover:not(:disabled) {
    background-color: #1976d2;
  }

  .submit-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }

  .form-group input:disabled {
    background-color: #f5f5f5;
    cursor: not-allowed;
  }
</style>
