<script lang="ts">
  import { toasts, removeToast } from '../stores/toast';
  import { ToastType } from '../types/toast';
  import { onMount } from 'svelte';

  // Subscribe to toast store
  $: activeToasts = $toasts;

  // Handle toast dismiss
  function dismissToast(id: string) {
    removeToast(id);
  }

  // Get appropriate icon for toast type
  function getToastIcon(type: string): string {
    switch (type) {
      case ToastType.ERROR:
        return '❌';
      case ToastType.SUCCESS:
        return '✅';
      case ToastType.WARNING:
        return '⚠️';
      case ToastType.INFO:
        return 'ℹ️';
      default:
        return 'ℹ️';
    }
  }

  // Get CSS class for toast type
  function getToastClass(type: string): string {
    switch (type) {
      case ToastType.ERROR:
        return 'toast-error';
      case ToastType.SUCCESS:
        return 'toast-success';
      case ToastType.WARNING:
        return 'toast-warning';
      case ToastType.INFO:
        return 'toast-info';
      default:
        return 'toast-info';
    }
  }
</script>

{#if activeToasts.length > 0}
  <div class="toast-container">
    {#each activeToasts as toast (toast.id)}
      <div
        class="toast {getToastClass(toast.type)}"
        role="alert"
        aria-live="polite"
      >
        <div class="toast-content">
          <span class="toast-icon" aria-hidden="true">
            {getToastIcon(toast.type)}
          </span>
          <span class="toast-message">{toast.message}</span>
        </div>
        <button
          class="toast-close"
          onclick={() => dismissToast(toast.id)}
          aria-label="Dismiss notification"
          type="button"
        >
          ✕
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 10000;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 400px;
    width: calc(100vw - 2rem);
    pointer-events: none;
  }

  @media (min-width: 480px) {
    .toast-container {
      width: 400px;
    }
  }

  .toast {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    backdrop-filter: blur(8px);
    pointer-events: auto;
    animation: slideIn 0.3s ease-out;
    border-left: 4px solid;
    background-color: rgba(255, 255, 255, 0.95);
  }

  .toast-content {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex: 1;
    min-width: 0;
  }

  .toast-icon {
    font-size: 1.2rem;
    flex-shrink: 0;
  }

  .toast-message {
    font-size: 0.9rem;
    line-height: 1.4;
    color: #333;
    word-wrap: break-word;
    flex: 1;
  }

  .toast-close {
    background: none;
    border: none;
    font-size: 1.2rem;
    cursor: pointer;
    color: #666;
    padding: 0.25rem;
    margin-left: 0.5rem;
    border-radius: 4px;
    flex-shrink: 0;
    transition: all 0.2s ease;
  }

  .toast-close:hover {
    background-color: rgba(0, 0, 0, 0.1);
    color: #333;
  }

  .toast-close:focus {
    outline: 2px solid #1e88e5;
    outline-offset: 2px;
  }

  /* Toast type specific styles */
  .toast-error {
    border-left-color: #dc2626;
    background-color: rgba(254, 242, 242, 0.95);
  }

  .toast-error .toast-message {
    color: #7f1d1d;
  }

  .toast-success {
    border-left-color: #16a34a;
    background-color: rgba(240, 253, 244, 0.95);
  }

  .toast-success .toast-message {
    color: #14532d;
  }

  .toast-warning {
    border-left-color: #d97706;
    background-color: rgba(255, 251, 235, 0.95);
  }

  .toast-warning .toast-message {
    color: #92400e;
  }

  .toast-info {
    border-left-color: #2563eb;
    background-color: rgba(239, 246, 255, 0.95);
  }

  .toast-info .toast-message {
    color: #1e40af;
  }

  /* Animations */
  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .toast {
    animation: slideIn 0.3s ease-out;
  }

  /* Mobile specific adjustments */
  @media (max-width: 480px) {
    .toast-container {
      top: 0.5rem;
      right: 0.5rem;
      left: 0.5rem;
      width: auto;
    }

    .toast {
      padding: 0.75rem;
    }

    .toast-message {
      font-size: 0.85rem;
    }

    .toast-icon {
      font-size: 1rem;
    }

    .toast-close {
      font-size: 1rem;
    }
  }

  /* Dark mode support (if needed in future) */
  @media (prefers-color-scheme: dark) {
    .toast {
      background-color: rgba(31, 41, 55, 0.95);
    }

    .toast-message {
      color: #f3f4f6;
    }

    .toast-close {
      color: #9ca3af;
    }

    .toast-close:hover {
      background-color: rgba(255, 255, 255, 0.1);
      color: #f3f4f6;
    }

    .toast-error {
      background-color: rgba(69, 10, 10, 0.95);
    }

    .toast-error .toast-message {
      color: #fca5a5;
    }

    .toast-success {
      background-color: rgba(5, 46, 22, 0.95);
    }

    .toast-success .toast-message {
      color: #86efac;
    }

    .toast-warning {
      background-color: rgba(69, 39, 7, 0.95);
    }

    .toast-warning .toast-message {
      color: #fcd34d;
    }

    .toast-info {
      background-color: rgba(7, 33, 82, 0.95);
    }

    .toast-info .toast-message {
      color: #93c5fd;
    }
  }
</style>