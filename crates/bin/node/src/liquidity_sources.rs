use std::marker::PhantomData;

use liquidity_source::LiquiditySource;
use nuts::traits::Unit;
use sqlx::PgPool;

use crate::{initialization::ProgramArguments, methods::Method};

#[derive(Debug, Clone)]
pub struct LiquiditySources<U: Unit> {
    #[cfg(feature = "starknet")]
    starknet: starknet_liquidity_source::StarknetLiquiditySource,
    _phantom_data: PhantomData<U>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "starknet")]
    #[error("failed to init starknet liquidity source: {0}")]
    Starknet(#[from] starknet_liquidity_source::Error),
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
        Ok(LiquiditySources {
            #[cfg(feature = "starknet")]
            starknet: starknet_liquidity_source::StarknetLiquiditySource::init(
                pg_pool,
                args.config
                    .ok_or(Error::MissingConfigFile(String::from("starknet")))?,
            )
            .await?,
            _phantom_data: PhantomData,
        })
    }

    pub fn get_liquidity_source(
        &self,
        method: Method,
    ) -> Option<impl LiquiditySource<Unit = starknet_types::Unit>> {
        match method {
            Method::Starknet => self.starknet(),
        }
    }
}

impl<U: Unit> LiquiditySources<U> {
    #[cfg(feature = "mock")]
    pub fn starknet(&self) -> Option<impl LiquiditySource<Unit = starknet_types::Unit>> {
        Some(liquidity_source::mock::MockLiquiditySource)
    }

    #[cfg(all(not(feature = "mock"), feature = "starknet"))]
    pub fn starknet(&self) -> Option<impl LiquiditySource<Unit = starknet_types::Unit>> {
        Some(self.starknet.clone())
    }
}
