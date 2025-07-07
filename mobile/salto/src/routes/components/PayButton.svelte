<script lang="ts">
  interface Props {
    onClick: () => void;
  }

  let { onClick }: Props = $props();
</script>

<button class="pay-button" onclick={onClick}>
  Pay
</button>

<style>
  .pay-button {
    background-color: #1e88e5;
    color: white;
    font-size: 1.2rem;
    font-weight: 600;
    padding: 0.8rem 3rem;
    border: none;
    border-radius: 50px;
    cursor: pointer;
    transition: background-color 0.2s, transform 0.1s;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
  }

  .pay-button:hover {
    background-color: #1976d2;
  }

  .pay-button:active {
    transform: scale(0.98);
    background-color: #1565c0;
  }
</style>