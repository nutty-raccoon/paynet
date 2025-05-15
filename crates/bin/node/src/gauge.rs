use std::time::Duration;

use opentelemetry::{KeyValue, metrics::Gauge};
use sqlx::{PgPool, Pool, Postgres};
use starknet_types::Unit;

pub struct DbMetricsObserver {
    pool: Pool<Postgres>,
    units: Vec<Unit>,
    gauge: Gauge<u64>,
}

impl DbMetricsObserver {
    pub fn new(pool: PgPool, units: Vec<Unit>, gauge: Gauge<u64>) -> Self {
        Self { pool, units, gauge }
    }

    async fn poll_metrics(&mut self) -> Result<(), anyhow::Error> {
        let mut conn = self.pool.acquire().await?;
        let gauges = db_node::gauge::get_all_gauge_metrics_by_units(&mut conn, &self.units).await?;

        for (unit, metrics) in gauges {
            self.gauge.record(
                metrics.pending_deposits.into(),
                &[
                    KeyValue::new("metric", "deposits.pending"),
                    KeyValue::new("unit", unit.clone()),
                ],
            );
            self.gauge.record(
                metrics.paid_deposits.into(),
                &[
                    KeyValue::new("metric", "deposits.paid"),
                    KeyValue::new("unit", unit.clone()),
                ],
            );
            self.gauge.record(
                metrics.issued_deposits.into(),
                &[
                    KeyValue::new("metric", "deposits.issued"),
                    KeyValue::new("unit", unit.clone()),
                ],
            );
            self.gauge.record(
                metrics.unpaid_withdrawals.into(),
                &[
                    KeyValue::new("metric", "withdrawals.unpaid"),
                    KeyValue::new("unit", unit.clone()),
                ],
            );
            self.gauge.record(
                metrics.pending_withdrawals.into(),
                &[
                    KeyValue::new("metric", "withdrawals.pending"),
                    KeyValue::new("unit", unit.clone()),
                ],
            );
            self.gauge.record(
                metrics.paid_withdrawals.into(),
                &[
                    KeyValue::new("metric", "withdrawals.paid"),
                    KeyValue::new("unit", unit.clone()),
                ],
            );
        }

        Ok(())
    }
}

pub async fn init_metrics_polling(mut observer: DbMetricsObserver) {
    loop {
        observer.poll_metrics().await.unwrap();
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
