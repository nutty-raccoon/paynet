use tonic::transport::Server;

#[cfg(not(feature = "tls"))]
pub fn build_server() -> Result<Server, anyhow::Error> {
    tracing::info!("🚀 Starting gRPC server...");

    Ok(tonic::transport::Server::builder())
}

#[cfg(feature = "tls")]
pub fn build_server() -> Result<Server, anyhow::Error> {
    const CERT_PATH_ENV_VAR: &str = "TLS_CERT_PATH";
    const KEY_PATH_ENV_VAR: &str = "TLS_KEY_PATH";

    // Get certificate and key paths from environment or use defaults
    let cert_path =
        std::env::var(CERT_PATH_ENV_VAR).unwrap_or_else(|_| "certs/cert.pem".to_string());
    let key_path = std::env::var(KEY_PATH_ENV_VAR).unwrap_or_else(|_| "certs/key.pem".to_string());

    // Load TLS certificates
    let cert = match std::fs::read(&cert_path) {
        Ok(cert) => {
            tracing::info!("✅ TLS certificate loaded successfully from {}", cert_path);
            cert
        }
        Err(e) => {
            eprintln!("❌ Failed to load TLS certificate:");
            eprintln!("   Certificate: {}", cert_path);
            eprintln!("   Error: {}", e);
            eprintln!();
            eprintln!("🚫 gRPC server cannot start without valid HTTPS certificates");

            #[cfg(debug_assertions)]
            {
                eprintln!();
                eprintln!("💡 To generate local certificates with mkcert:");
                eprintln!("   1. Install mkcert: https://github.com/FiloSottile/mkcert");
                eprintln!("   2. Run: mkcert -install");
                eprintln!("   3. Run: mkdir -p certs");
                eprintln!(
                    "   4. Run: mkcert -key-file certs/key.pem -cert-file certs/cert.pem localhost 127.0.0.1 ::1"
                );
                eprintln!();
            }
            return Err(anyhow::anyhow!("Failed to load TLS certificate: {}", e));
        }
    };

    let key = match std::fs::read(&key_path) {
        Ok(key) => {
            tracing::info!("✅ TLS private key loaded successfully from {}", key_path);
            key
        }
        Err(e) => {
            eprintln!("❌ Failed to load TLS private key:");
            eprintln!("   Private key: {}", key_path);
            eprintln!("   Error: {}", e);
            return Err(anyhow::anyhow!("Failed to load TLS private key: {}", e));
        }
    };

    let identity = tonic::transport::Identity::from_pem(cert, key);
    let tls_config = tonic::transport::ServerTlsConfig::new().identity(identity);

    tracing::info!("🔒 Starting gRPC server with TLS...");
    tracing::info!("📜 Certificate: {}", cert_path);
    tracing::info!("🔑 Private key: {}", key_path);

    let server = tonic::transport::Server::builder().tls_config(tls_config)?;

    Ok(server)
}
