<script lang="ts">
  import { URDecoder } from "@gandlaf21/bc-ur";
  import QrCodeScanner from "./components/QrCodeScanner.svelte";
  import Portal from "../components/Portal.svelte";
  import { receive_wads } from "../../commands";

  interface Props {
    onSuccess: () => void;
    onCancell: () => void;
  }

  let { onCancell, onSuccess }: Props = $props();

  let percentageEstimate = $state("");
  let decoder = $state(new URDecoder());
  let paused = $state(false);

  function onCodeDetected(decodedText: string) {
    decoder.receivePart(decodedText);
    const estimatedPercentComplete = decoder.estimatedPercentComplete();
    percentageEstimate = (estimatedPercentComplete * 100).toFixed(0) + "%";

    if (decoder.isComplete()) {
      paused = true;
      if (decoder.isSuccess()) {
        // Get the UR representation of the message
        const ur = decoder.resultUR();
        // Decode the CBOR message to a Buffer
        const decoded = ur.decodeCBOR();
        receive_wads(decoded.toString());
        onSuccess();
      } else {
        // log and handle the error
        const error = decoder.resultError();
        console.log("Error found while decoding", error);
      }
    }
  }
</script>

<Portal isOpen={true} onClose={onCancell} backgroundColor="rgba(0, 0, 0, 0.95)">
  {#snippet children()}
    <div class="scan-content">
      <p class="scan-instructions">Point your camera at a QR code</p>

      <QrCodeScanner {onCodeDetected} {paused} />

      {#if percentageEstimate}
        <div class="scan-result">
          <h3>Scanned:</h3>
          <p>{percentageEstimate}</p>
        </div>
      {/if}

      <button class="cancel-button" onclick={onCancell}>Cancel</button>
    </div>
  {/snippet}
</Portal>

<style>
  .scan-content {
    color: white;
    text-align: center;
  }

  .scan-instructions {
    margin: 0 0 2rem 0;
    font-size: 1rem;
    opacity: 0.8;
  }

  .scan-result {
    background-color: rgba(255, 255, 255, 0.1);
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 2rem;
  }

  .scan-result h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.2rem;
    color: #4caf50;
  }

  .scan-result p {
    margin: 0;
    word-break: break-all;
    font-family: monospace;
    font-size: 0.9rem;
  }

  .cancel-button {
    background-color: #d32f2f;
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
    padding: 0.8rem 2rem;
    border: none;
    border-radius: 50px;
    cursor: pointer;
    transition:
      background-color 0.2s,
      transform 0.1s;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
    width: 100%;
  }

  .cancel-button:hover {
    background-color: #b71c1c;
  }

  .cancel-button:active {
    transform: scale(0.98);
    background-color: #8e0000;
  }
</style>
