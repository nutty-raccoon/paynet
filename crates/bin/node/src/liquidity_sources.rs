use liquidity_source::LiquiditySource;
use sqlx::PgPool;

use crate::{initialization::ProgramArguments, methods::Method};

#[derive(Debug, Clone)]
pub struct LiquiditySources {
    #[cfg(feature = "starknet")]
    starknet: starknet_liquidity_source::StarknetLiquiditySource,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "starknet")]
    #[error("failed to init starknet liquidity source: {0}")]
    Starknet(#[from] starknet_liquidity_source::Error),
    #[error("failed to acquire db connection: {0}")]
    SqlxAcquire(#[from] sqlx::Error),
}

impl LiquiditySources {
    #[allow(unused_variables)]
    pub async fn init(pg_pool: PgPool, args: ProgramArguments) -> Result<LiquiditySources, Error> {
        Ok(LiquiditySources {
            #[cfg(feature = "starknet")]
            starknet: starknet_liquidity_source::StarknetLiquiditySource::init(
                pg_pool.acquire().await?,
                args.config,
            )
            .await?,
        })
    }

    pub fn get_liquidity_source(&self, method: Method) -> Option<impl LiquiditySource> {
        match method {
            Method::Starknet => self.starknet(),
        }
    }
}

impl LiquiditySources {
    #[cfg(feature = "mock")]
    pub fn starknet(&self) -> Option<impl LiquiditySource> {
        Some(liquidity_source::mock::MockLiquiditySource)
    }

    #[cfg(all(not(feature = "mock"), feature = "starknet"))]
    pub fn starknet(&self) -> Option<impl LiquiditySource> {
        Some(self.starknet.clone())
    }
}
