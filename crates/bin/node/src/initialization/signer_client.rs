use signer::SignerClient;
use tonic::transport::Channel;

use super::Error;

pub async fn connect_to_signer(signer_url: String) -> Result<SignerClient<Channel>, Error> {
    let signer_client = signer::SignerClient::connect(signer_url)
        .await
        .map_err(Error::SignerConnection)?;

    Ok(signer_client)
}
