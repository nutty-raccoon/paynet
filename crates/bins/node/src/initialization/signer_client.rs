use tonic::transport::Channel;
use tower_otel::trace;
use tracing::Level;

use crate::app_state::SignerClient;

use super::Error;

pub async fn connect_to_signer(signer_url: String) -> Result<SignerClient, Error> {
    let channel = Channel::builder(signer_url.parse()?)
        .connect()
        .await
        .map_err(Error::SignerConnection)?;
    let channel = tower::ServiceBuilder::new()
        .layer(trace::GrpcLayer::client(Level::INFO))
        .service(channel);

    Ok(signer::SignerClient::new(channel))
}
