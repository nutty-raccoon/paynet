use std::marker::PhantomData;

use liquidity_source::LiquiditySource;
use nuts::traits::Unit;
use sqlx::PgPool;

use crate::{initialization::{ProgramArguments, nuts_settings::UnifiedUnit}, methods::Method};

/// Enum to represent different liquidity sources with their specific unit types
#[derive(Debug, Clone)]
pub enum AnyLiquiditySource {
    #[cfg(feature = "starknet")]
    Starknet(starknet_liquidity_source::StarknetLiquiditySource),
    #[cfg(feature = "ethereum")]
    Ethereum(ethereum_liquidity_source::EthereumLiquiditySource),
}

#[derive(Debug, Clone)]
pub struct LiquiditySources<U: Unit> {
    #[cfg(feature = "starknet")]
    starknet: starknet_liquidity_source::StarknetLiquiditySource,
    #[cfg(feature = "ethereum")]
    ethereum: ethereum_liquidity_source::EthereumLiquiditySource,
    _phantom_data: PhantomData<U>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "starknet")]
    #[error("failed to init starknet liquidity source: {0}")]
    Starknet(#[from] starknet_liquidity_source::Error),
    #[cfg(feature = "ethereum")]
    #[error("failed to init ethereum liquidity source: {0}")]
    Ethereum(#[from] ethereum_liquidity_source::Error),
    #[error("failed to acquire db connection: {0}")]
    SqlxAcquire(#[from] sqlx::Error),
    #[cfg(not(feature = "mock"))]
    #[error("feature {0} requires the arg `--config` to be given a value")]
    MissingConfigFile(String),
}

impl<U: Unit> LiquiditySources<U> {
    #[allow(unused_variables)]
    pub async fn init(
        pg_pool: PgPool,
        args: ProgramArguments,
    ) -> Result<LiquiditySources<U>, Error> {
        #[cfg(feature = "starknet")]
        let starknet = {
            #[cfg(not(feature = "mock"))]
            {
                starknet_liquidity_source::StarknetLiquiditySource::init(
                    pg_pool.clone(),
                    args.config
                        .clone()
                        .ok_or(Error::MissingConfigFile(String::from("starknet")))?,
                )
                .await?
            }
            #[cfg(feature = "mock")]
            {
                starknet_liquidity_source::StarknetLiquiditySource::new()
            }
        };

        #[cfg(feature = "ethereum")]
        let ethereum = {
            #[cfg(not(feature = "mock"))]
            {
                ethereum_liquidity_source::EthereumLiquiditySource::init(
                    pg_pool.clone(),
                    args.ethereum_config
                        .ok_or(Error::MissingConfigFile(String::from("ethereum")))?,
                )
                .await?
            }
            #[cfg(feature = "mock")]
            {
                ethereum_liquidity_source::EthereumLiquiditySource::new()
            }
        };

        Ok(LiquiditySources {
            #[cfg(feature = "starknet")]
            starknet,
            #[cfg(feature = "ethereum")]
            ethereum,
            _phantom_data: PhantomData,
        })
    }

    pub fn get_liquidity_source(&self, method: Method) -> Option<AnyLiquiditySource> {
        match method {
            #[cfg(feature = "starknet")]
            Method::Starknet => Some(AnyLiquiditySource::Starknet(self.starknet.clone())),
            #[cfg(feature = "ethereum")]
            Method::Ethereum => Some(AnyLiquiditySource::Ethereum(self.ethereum.clone())),
            #[cfg(not(feature = "starknet"))]
            Method::Starknet => None,
            #[cfg(not(feature = "ethereum"))]
            Method::Ethereum => None,
        }
    }
}
