<script lang="ts">
  import { goto } from "$app/navigation";

  interface Props {
    isMobile: boolean;
    onError: (error: string) => void;
  }

  let { isMobile, onError }: Props = $props();

  function handleClick() {
    if (isMobile) {
      goto("/scan");
    } else {
      onError("qrcode scan only available on mobile");
    }
  }
</script>

<button class="receive-button" onclick={handleClick}> Receive </button>

<style>
  .receive-button {
    background-color: #2e7d32;
    color: white;
    font-size: 1.2rem;
    font-weight: 600;
    padding: 0.8rem 3rem;
    border: none;
    border-radius: 50px;
    cursor: pointer;
    transition:
      background-color 0.2s,
      transform 0.1s;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  }

  .receive-button:hover {
    background-color: #1b5e20;
  }

  .receive-button:active {
    transform: scale(0.98);
    background-color: #0d4814;
  }
</style>
