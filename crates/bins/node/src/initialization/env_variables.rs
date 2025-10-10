use std::env::VarError;

use super::Error;

pub fn read_env_variables() -> Result<EnvVariables, Error> {
    // Only if we are in debug mode, we allow loading env variable from a .env file
    #[cfg(debug_assertions)]
    {
        let _ = dotenvy::from_filename("node.env")
            .inspect_err(|e| tracing::error!("dotenvy initialization failed: {e}"));
    }

    let pg_url = std::env::var("PG_URL").map_err(|e| Error::Env("PG_URL", e))?;
    let signer_url = std::env::var("SIGNER_URL").map_err(|e| Error::Env("SIGNER_URL", e))?;
    let grpc_port = std::env::var("GRPC_PORT")
        .map_err(|e| Error::Env("GRPC_PORT", e))?
        .parse()
        .map_err(Error::ParseInt)?;
    #[cfg(feature = "rest")]
    let rest_port = std::env::var("REST_PORT")
        .unwrap_or_else(|_| "80".to_string()) // Default REST port
        .parse()
        .map_err(Error::ParseInt)?;
    let quote_ttl = match std::env::var("QUOTE_TTL") {
        Ok(v) => Some(v.parse().map_err(Error::ParseInt)?),
        Err(VarError::NotPresent) => None,
        Err(e) => return Err(Error::Env("QUOTE_TTL", e)),
    };

    #[cfg(feature = "tls")]
    let tls_cert_path =
        std::env::var("TLS_CERT_PATH").map_err(|e| Error::Env("TLS_CERT_PATH", e))?;
    #[cfg(feature = "tls")]
    let tls_key_path = std::env::var("TLS_KEY_PATH").map_err(|e| Error::Env("TLS_KEY_PATH", e))?;

    Ok(EnvVariables {
        pg_url,
        signer_url,
        grpc_port,
        #[cfg(feature = "rest")]
        rest_port,
        quote_ttl,
        #[cfg(feature = "tls")]
        tls_cert_path,
        #[cfg(feature = "tls")]
        tls_key_path,
    })
}

#[derive(Debug)]
pub struct EnvVariables {
    pub pg_url: String,
    pub signer_url: String,
    pub grpc_port: u16,
    #[cfg(feature = "rest")]
    pub rest_port: u16,
    pub quote_ttl: Option<u64>,
    #[cfg(feature = "tls")]
    pub tls_cert_path: String,
    #[cfg(feature = "tls")]
    pub tls_key_path: String,
}
