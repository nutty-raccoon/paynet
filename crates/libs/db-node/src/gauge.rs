use num_traits::ToPrimitive;
use nuts::{Amount, traits::Unit};
use sqlx::Error;
use sqlx::PgConnection;

pub async fn get_all_gauge_metrics_by_units<U: Unit>(
    conn: &mut PgConnection,
    units: &[U],
) -> Result<Vec<(String, GaugeMetrics)>, Error> {
    // Convert each unit to string
    let unit_strs: Vec<String> = units.iter().map(|u| u.to_string()).collect();

    // Query data for all units at once
    let records = sqlx::query!(
        r#"
            SELECT 
                unit AS "unit!",
                (SELECT COALESCE(SUM(amount), 0) FROM mint_quote WHERE unit = mq.unit AND state = 'UNPAID') AS "pending_deposits!",
                (SELECT COALESCE(SUM(amount), 0) FROM mint_quote WHERE unit = mq.unit AND state = 'PAID') AS "paid_deposits!",
                (SELECT COALESCE(SUM(amount), 0) FROM mint_quote WHERE unit = mq.unit AND state = 'ISSUED') AS "issued_deposits!",
                (SELECT COALESCE(SUM(amount), 0) FROM melt_quote WHERE unit = mq.unit AND state = 'UNPAID') AS "unpaid_withdrawals!",
                (SELECT COALESCE(SUM(amount), 0) FROM melt_quote WHERE unit = mq.unit AND state = 'PENDING') AS "pending_withdrawals!",
                (SELECT COALESCE(SUM(amount), 0) FROM melt_quote WHERE unit = mq.unit AND state = 'PAID') AS "paid_withdrawals!"
            FROM (SELECT DISTINCT unit FROM unnest($1::text[]) AS unit) mq
        "#,
        &unit_strs
    )
    .fetch_all(conn)
    .await?;

    let mut results = Vec::with_capacity(units.len());
    for record in records {
        results.push((
            record.unit,
            GaugeMetrics {
                pending_deposits: Amount::from(record.pending_deposits.to_u64().unwrap()),
                paid_deposits: Amount::from(record.paid_deposits.to_u64().unwrap()),
                issued_deposits: Amount::from(record.issued_deposits.to_u64().unwrap()),
                unpaid_withdrawals: Amount::from(record.unpaid_withdrawals.to_u64().unwrap()),
                pending_withdrawals: Amount::from(record.pending_withdrawals.to_u64().unwrap()),
                paid_withdrawals: Amount::from(record.paid_withdrawals.to_u64().unwrap()),
            },
        ));
    }

    Ok(results)
}

/// Structure to hold all the gauge metrics for a specific unit
#[derive(Debug, Clone)]
pub struct GaugeMetrics {
    pub pending_deposits: Amount,
    pub paid_deposits: Amount,
    pub issued_deposits: Amount,
    pub unpaid_withdrawals: Amount,
    pub pending_withdrawals: Amount,
    pub paid_withdrawals: Amount,
}
