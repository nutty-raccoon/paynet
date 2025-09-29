export async function checkMacOSCameraPermission(): Promise<boolean> {
  try {
    const { checkCameraPermission, requestCameraPermission } = await import("tauri-plugin-macos-permissions-api");
    
    const authorized = await checkCameraPermission();
    if (!authorized) {
      await requestCameraPermission();
      return await checkCameraPermission();
    }
    return authorized;
  } catch (error) {
    console.warn("macOS permissions not available:", error);
    return false; // Assume permission granted if module not available
  }
}
