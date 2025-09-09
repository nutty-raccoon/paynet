<script lang="ts">
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";

  interface Props {
    seedPhrase: string;
    showCopyHint?: boolean;
  }

  let { seedPhrase, showCopyHint = true }: Props = $props();
</script>

<button
  class="seed-phrase-box"
  onclick={async () => await writeText(seedPhrase)}
>
  <p class="seed-phrase-text">{seedPhrase}</p>
</button>

<style>
  .seed-phrase-box {
    background-color: #f8f9fa;
    border: 2px solid #e9ecef;
    border-radius: 12px;
    padding: 1.5rem;
    margin: 1.5rem 0;
    cursor: pointer;
    transition: all 0.2s ease;
    position: relative;
    overflow: hidden;
    width: 100%;
  }

  .seed-phrase-box::before {
    content: "📋 Click to copy";
    position: absolute;
    top: 0.5rem;
    right: 0.75rem;
    font-size: 0.75rem;
    color: #6b7280;
    opacity: 0.7;
    pointer-events: none;
  }

  .seed-phrase-box:hover {
    background-color: #e3f2fd;
    border-color: #1e88e5;
    box-shadow: 0 4px 12px rgba(30, 136, 229, 0.15);
    transform: translateY(-1px);
  }

  .seed-phrase-box:hover::before {
    color: #1e88e5;
    opacity: 1;
  }

  .seed-phrase-box:active {
    transform: translateY(0px) scale(0.98);
    background-color: #bbdefb;
    border-color: #1976d2;
    box-shadow: 0 2px 6px rgba(30, 136, 229, 0.2);
  }

  .seed-phrase-box:focus {
    outline: none;
    box-shadow: 0 0 0 3px rgba(30, 136, 229, 0.3);
  }

  .seed-phrase-text {
    font-family: "Courier New", monospace;
    font-size: 1rem;
    color: #0f0f0f;
    line-height: 1.6;
    margin: 0;
    word-break: break-word;
  }
</style>