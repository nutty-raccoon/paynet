<script lang="ts">
  interface Props {
    onQRCodeChoice: () => void;
    onPasteChoice: () => void;
  }

  let { onQRCodeChoice, onPasteChoice }: Props = $props();
</script>

<div class="payment-method-choice">
  <h4>Choose Receiving Method</h4>
  <div class="method-buttons">
    <button class="method-button qr-button" onclick={onQRCodeChoice}>
      <div class="method-icon">📱</div>
      <span>QR Code</span>
      <small>Scan</small>
    </button>
    <button class="method-button paste-button" onclick={onPasteChoice}>
      <div class="method-icon">📋</div>
      <span>Paste</span>
      <small>Paste from clipboard</small>
    </button>
  </div>
</div>

<style>
  .payment-method-choice {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 2rem;
    background-color: #f8f9fa;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  h4 {
    margin: 0;
    color: #333;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .method-buttons {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    justify-content: center;
  }

  .method-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    padding: 1.5rem;
    border: 2px solid transparent;
    border-radius: 12px;
    background-color: white;
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 120px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .method-button:hover {
    border-color: #1e88e5;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(30, 136, 229, 0.2);
  }

  .method-button:active {
    transform: translateY(0);
  }

  .method-icon {
    font-size: 2rem;
  }

  .method-button span {
    font-weight: 600;
    color: #333;
  }

  .method-button small {
    color: #666;
    font-size: 0.8rem;
  }

  .qr-button:hover {
    background-color: #e8f5e8;
  }

  .paste-button:hover {
    background-color: #fff3e0;
  }
</style>
