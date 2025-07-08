<script lang="ts">
  import {
    scan,
    textRecord,
    write,
    type ScanKind,
  } from "@tauri-apps/plugin-nfc";
  import { onMount } from "svelte";

  interface Props {
    onClose: () => void;
    isReceiving: boolean;
  }

  let { onClose, isReceiving }: Props = $props();

  async function scanNFC() {
    const scanType: ScanKind = {
      type: "ndef",
    };

    const options = {
      keepSessionAlive: true,
      // configure the messages displayed in the "Scan NFC" dialog on iOS
      message: "Scan a NFC tag",
      successMessage: "NFC tag successfully scanned",
    };

    console.log("SCANNING");
    const tag = await scan(scanType, options);
    console.log(tag);
    alert("GOT TAG");
  }
  async function writeNFC() {
    const scanType: ScanKind = {
      type: "ndef",
    };

    const options = {
      keepSessionAlive: true,
      // configure the messages displayed in the "Scan NFC" dialog on iOS
      message: "Scan a NFC tag",
      successMessage: "NFC tag successfully scanned",
    };

    console.log("looking for tag");
    const tag = await scan(scanType, options);
    console.log("got tag", JSON.stringify(tag));
    await write([textRecord("Tauri is awesome!")]);
  }

  onMount(() => {
    if (isReceiving) {
      scanNFC()
        .then(() => {
          alert("NFC scanned");
        })
        .catch(() => {
          alert("NFC not scanned");
        });
    } else {
      writeNFC()
        .then(() => {
          alert("NFC written");
        })
        .catch(() => {
          alert("NFC not written");
        });
    }
  });
</script>

<div class="nfc-modal">
  <div class="nfc-icon">📱</div>
  <h4>NFC Payment</h4>
  <p class="nfc-instruction">Approach device to pay</p>
  <button class="cancel-button" onclick={onClose}>Cancel</button>
</div>

<style>
  .nfc-modal {
    text-align: center;
    padding: 2rem 1rem;
  }

  .nfc-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  .nfc-modal h4 {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
    color: #333;
    font-weight: 600;
  }

  .nfc-instruction {
    font-size: 1.1rem;
    color: #666;
    margin-bottom: 2rem;
    line-height: 1.5;
  }

  .cancel-button {
    padding: 0.8rem 2rem;
    background-color: #666;
    color: white;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background-color 0.2s;
    font-size: 1rem;
  }

  .cancel-button:hover {
    background-color: #555;
  }
</style>
