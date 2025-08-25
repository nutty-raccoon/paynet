fn main() {
    tauri_build::build();

    #[cfg(feature = "tls-local-mkcert")]
    if let Err(e) = copy_mkcert_root_ca() {
        println!(
            "cargo:error=Failed to copy mkcert root CA certificate: {}",
            e
        );
        println!("cargo:error=Make sure `mkcert` is properly installed");
    }
}

#[cfg(feature = "tls-local-mkcert")]
fn copy_mkcert_root_ca() -> Result<(), Box<dyn std::error::Error>> {
    use std::path::PathBuf;
    use std::process::Command;

    // Execute mkcert -CAROOT command
    let output = Command::new("mkcert")
        .args(["-CAROOT"])
        .output()
        .map_err(|e| format!("Failed to execute mkcert command: {}", e))?;

    // Check if the command was successful
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("mkcert command failed: {}", stderr).into());
    }

    // Parse the output to get the CA root path
    let path_string = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in mkcert output: {}", e))?
        .trim()
        .to_string();

    let mut src = PathBuf::from(path_string);
    src.push("rootCA.pem");

    // Check if the source file exists
    if !src.exists() {
        return Err(format!("Root CA file not found at: {}", src.display()).into());
    }

    // Create destination directory
    let mut dest = PathBuf::from("./certs");
    std::fs::create_dir_all(&dest)
        .map_err(|e| format!("Failed to create certs directory: {}", e))?;

    dest.push("rootCA.pem");

    // Copy the file
    std::fs::copy(&src, &dest).map_err(|e| {
        format!(
            "Failed to copy {} to {}: {}",
            src.display(),
            dest.display(),
            e
        )
    })?;

    println!(
        "cargo:info=Successfully copied mkcert root CA to {}",
        dest.display()
    );
    Ok(())
}
