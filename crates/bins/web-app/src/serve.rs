use std::net::SocketAddr;

use axum::Router;

#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;

#[cfg(feature = "tls")]
pub async fn serve(app: Router, bind_address: SocketAddr) {
    // Get certificate and key paths from environment or use defaults
    let cert_path = std::env::var("CERT_PATH").unwrap_or_else(|_| "certs/cert.pem".to_string());
    let key_path = std::env::var("KEY_PATH").unwrap_or_else(|_| "certs/key.pem".to_string());

    // Create TLS config - this will fail if certificates don't exist
    let tls_config = match RustlsConfig::from_pem_file(&cert_path, &key_path).await {
        Ok(config) => {
            println!("✅ TLS certificates loaded successfully");
            config
        }
        Err(e) => {
            eprintln!("❌ Failed to load TLS certificates:");
            eprintln!("   Certificate: {}", cert_path);
            eprintln!("   Private key: {}", key_path);
            eprintln!("   Error: {}", e);
            eprintln!();
            eprintln!("🚫 Server cannot start without valid HTTPS certificates");

            #[cfg(debug_assertions)]
            {
                eprintln!();
                eprintln!("💡 To generate local certificates with mkcert:");
                eprintln!("   1. Install mkcert: https://github.com/FiloSottile/mkcert");
                eprintln!("   2. Run: mkcert -install");
                eprintln!("   3. Run: mkdir -p certs");
                eprintln!("   4. Run: mkcert -key-file certs/key.pem -cert-file certs/cert.pem localhost 127.0.0.1 ::1");
                eprintln!();
            }
            panic!();
        }
    };

    println!("🔒 Starting HTTPS server...");
    println!("📜 Certificate: {}", cert_path);
    println!("🔑 Private key: {}", key_path);
    println!("🚀 Binding to: https://{}", bind_address);

    // Serve
    axum_server::bind_rustls(bind_address, tls_config)
        .serve(app.into_make_service())
        .await
        .expect("the server should run")
}

#[cfg(not(feature = "tls"))]
pub async fn serve(app: Router, bind_address: SocketAddr) {
    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .expect("should be able to listen");

    println!("🚀 Server running on http://{}", bind_address);
    axum::serve(listener, app)
        .await
        .expect("the server should run");
}
