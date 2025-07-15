<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  interface Props {
    onClose: () => void;
    title?: string;
    showCloseButton?: boolean;
    maxWidth?: string;
    backgroundColor?: string;
    children: import("svelte").Snippet;
  }

  let {
    onClose,
    title = "",
    showCloseButton = true,
    maxWidth = "400px",
    backgroundColor = "white",
    children,
  }: Props = $props();

  onMount(() => {
    // Prevent body scroll
    document.body.style.overflow = "hidden";
  });

  onDestroy(() => {
    // Restore body scroll
    document.body.style.overflow = "";
  });

  const handleClose = () => {
    onClose();
  };
</script>

<div class="portal-overlay">
  <div
    class="portal-content"
    style="max-width: {maxWidth}; background-color: {backgroundColor};"
  >
    {#if title || showCloseButton}
      <div class="portal-header">
        {#if title}
          <h2>{title}</h2>
        {/if}
        {#if showCloseButton}
          <button class="close-button" onclick={handleClose}>✕</button>
        {/if}
      </div>
    {/if}

    <div class="portal-body">
      {@render children()}
    </div>
  </div>
</div>

<style>
  .portal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    z-index: 9999;
  }

  .portal-content {
    border-radius: 16px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    display: flex;
    flex-direction: column;
  }

  .portal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 1.5rem 0;
    border-bottom: 1px solid #eee;
    margin-bottom: 1.5rem;
  }

  .portal-header h2 {
    margin: 0;
    font-size: 1.5rem;
    color: #333;
    font-weight: 600;
  }

  .close-button {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: #666;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    transition: background-color 0.2s;
    line-height: 1;
  }

  .close-button:hover {
    background-color: #f0f0f0;
  }

  .portal-body {
    flex: 1;
    padding: 0 0.75rem 1.5rem;
  }
</style>
