<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { beforeNavigate, goto } from "$app/navigation";
  import {
    scan,
    Format,
    checkPermissions,
    requestPermissions,
    cancel,
  } from "@tauri-apps/plugin-barcode-scanner";
  import { URDecoder } from "@gandlaf21/bc-ur";
  import jsQR from "jsqr";

  let scanningInProgress = $state(false);
  let scanResult = $state("");
  let originalHtmlStyle = "";

  //  async function startScaningCodes() {
  //    const decoder = new URDecoder();
  //
  //    do {
  //      // Scan the part from a QRCode
  //      // the part should look like this:
  //      // ur:bytes/1-9/lpadascfadaxcywenbpljkhdcahkadaemejtswhhylkepmykhhtsytsnoyoyaxaedsuttydmmhhpktpmsrjtdkgslpgh
  //      const part = await uncheckedScanQrCode();
  //
  //      // register the new part with the decoder
  //      decoder.receivePart(part);
  //
  //      const x = decoder.estimatedPercentComplete();
  //      scanResult = x.toString();
  //      // check if all the necessary parts have been received to successfully decode the message
  //    } while (!decoder.isComplete());
  //
  //    // If no error has been found
  //    if (decoder.isSuccess()) {
  //      // Get the UR representation of the message
  //      const ur = decoder.resultUR();
  //
  //      // Decode the CBOR message to a Buffer
  //      const decoded = ur.decodeCBOR();
  //
  //      // get the original message, assuming it was a JSON object
  //      const originalMessage = JSON.parse(decoded.toString());
  //      scanResult = originalMessage;
  //    } else {
  //      // log and handle the error
  //      const error = decoder.resultError();
  //      console.log("Error found while decoding", error);
  //    }
  //  }

  async function scanQRCode() {
    try {
      const permission = await checkPermissions();
      if (permission == "granted") {
        return await uncheckedScanQrCode();
      } else {
        const permission = await requestPermissions();
        if (permission == "granted") {
          return await uncheckedScanQrCode();
        } else {
          return "Permission denied";
        }
      }
    } catch (error) {
      console.error("QR code scanning failed:", error);
      return JSON.stringify(error);
    }
  }

  async function uncheckedScanQrCode() {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: "environment", focusDistance: 0.5 },
      });
      const track = stream.getVideoTracks()[0];
      var video = document.getElementById("video") as HTMLVideoElement;
      var canvas = document.getElementById("canvas") as HTMLCanvasElement;
      var context = canvas.getContext("2d") as CanvasRenderingContext2D;

      video.srcObject = stream;
      video.play();

      setInterval(function () {
        context.drawImage(video, 0, 0, 900, 900);
        var imageData = context.getImageData(0, 0, 900, 900);
        var code = jsQR(imageData.data, imageData.width, imageData.height);
        if (code) {
          alert(code.data);
        } else {
          //   result.textContent = "waitting";
        }
      }, 500);
    } catch (e) {
      console.log(e);
    }
  }

  // Reactive statement to handle background color changes
  $effect(() => {
    if (typeof document !== "undefined") {
      const html = document.documentElement;

      if (scanningInProgress) {
        // Store original style
        originalHtmlStyle = html.style.backgroundColor || "";

        // Set it to transparent
        html.style.backgroundColor = "transparent";
      } else {
        // Restore original style
        html.style.backgroundColor = originalHtmlStyle;
      }
    }
  });

  async function cancelScanning() {
    if (scanningInProgress) {
      try {
        await cancel();
        scanningInProgress = false;
        return "Scanning cancelled";
      } catch (error) {
        console.error("Failed to cancel scanning:", error);
        scanningInProgress = false;
        return "Cancel failed";
      }
    }
    return "No scanning in progress";
  }

  async function handleCancel() {
    await cancelScanning();
    goto("/");
  }

  onMount(() => {
    // Start scanning immediately when page loads
    scanQRCode();
  });

  onDestroy(() => {
    // Restore original styles if component is destroyed during scanning
    if (typeof document !== "undefined") {
      const html = document.documentElement;

      html.style.backgroundColor = originalHtmlStyle;
    }
  });
</script>

<div class="scan-container">
  <div class="scan-overlay">
    <h1>Scanning QR Code...</h1>
    <p>Point your camera at a QR code</p>
    <video id="video" width="300" height="300" autoplay></video>
    <canvas id="canvas" width="1200" height="1200" style="display:none;"
    ></canvas>

    {#if scanResult}
      <div class="scan-result">
        <h2>Scanned:</h2>
        <p>{scanResult}</p>
      </div>
    {/if}

    <button class="cancel-button" onclick={handleCancel}> Cancel </button>
  </div>
</div>

<style>
  .scan-container {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .scan-overlay {
    background-color: rgba(0, 0, 0, 0.8);
    color: white;
    padding: 2rem;
    border-radius: 12px;
    text-align: center;
    max-width: 320px;
    width: 90%;
  }

  .scan-overlay h1 {
    margin: 0 0 1rem 0;
    font-size: 1.5rem;
    font-weight: 600;
  }

  .scan-overlay p {
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

  .scan-result h2 {
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
  }

  .cancel-button:hover {
    background-color: #b71c1c;
  }

  .cancel-button:active {
    transform: scale(0.98);
    background-color: #8e0000;
  }
</style>
