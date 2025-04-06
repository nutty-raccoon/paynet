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

    Ok(EnvVariables {
        pg_url,
        signer_url,
        grpc_port,
    })
}

#[derive(Debug)]
pub struct EnvVariables {
    pub pg_url: String,
    pub signer_url: String,
    pub grpc_port: u16,
}
