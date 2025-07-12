use futures_util::StreamExt;
use nuts::{Amount, nut01::PublicKey, nut02::KeysetId, traits::Unit};
use sqlx::{PgConnection, Row};

use crate::Error;

/// Will return true if this secret has already been signed by us
pub async fn is_any_blind_message_already_used(
    conn: &mut PgConnection,
    blind_secrets: impl Iterator<Item = PublicKey>,
) -> Result<bool, sqlx::Error> {
    let ys: Vec<_> = blind_secrets.map(|pk| pk.to_bytes().to_vec()).collect();

    let record = sqlx::query!(
        r#"SELECT EXISTS (
            SELECT * FROM blind_signature WHERE y = ANY($1)
        ) AS "exists!";"#,
        &ys
    )
    .fetch_one(conn)
    .await?;

    Ok(record.exists)
}

pub async fn sum_amount_of_unit_in_circulation<U: Unit>(
    conn: &mut PgConnection,
    unit: U,
) -> Result<Amount, Error> {
    let record = sqlx::query!(
        r#"
            SELECT SUM(amount) AS "sum!: i64" FROM blind_signature 
            INNER JOIN keyset ON blind_signature.keyset_id = keyset.id
            WHERE keyset.unit = $1;
        "#,
        &unit.to_string()
    )
    .fetch_one(conn)
    .await?;

    let amount = Amount::from_i64_repr(record.sum);

    Ok(amount)
}

#[derive(Debug)]
pub struct RestoreFromDbResponse {
    pub amount: Amount,
    pub keyset_id: KeysetId,
    pub blinded_secret: PublicKey,
    pub blind_signature: PublicKey,
}

pub async fn get_by_blind_secrets(
    conn: &mut PgConnection,
    blind_secrets: impl ExactSizeIterator<Item = PublicKey>,
) -> Result<Vec<RestoreFromDbResponse>, Error> {
    let n_blind_secrets = blind_secrets.len();

    // Build query
    // We use the second value of the tupple as an ordering index
    // This way all values are guaranteed to be returned in the same order
    //  as the blind secrets used to fetch them
    let placeholders = (0..n_blind_secrets)
        .map(|i| {
            let i = i + 1;
            format!("(${}, {})", i, i)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        r#"
        SELECT amount, keyset_id, c, y FROM blind_signature
        JOIN (
          VALUES ({})
        ) AS v(y, position) ON blind_signature.y = v.y
        ORDER BY v.position;"#,
        placeholders
    );
    let mut query = sqlx::query(sql.as_str());
    for output in blind_secrets.into_iter() {
        query = query.bind(output.to_bytes());
    }

    // Get a stream of rows from db
    let mut rows_stream = query.fetch(conn);

    // Process and cast
    let mut ret = Vec::with_capacity(n_blind_secrets);
    while let Some(row) = rows_stream.next().await {
        let row = row?;
        let amount = row.try_get::<i64, _>(0)?;
        let keyset_id = row.try_get::<i64, _>(1)?;
        let c = row.try_get::<&[u8], _>(2)?;
        let y = row.try_get::<&[u8], _>(3)?;

        ret.push(RestoreFromDbResponse {
            amount: Amount::from_i64_repr(amount),
            keyset_id: keyset_id
                .try_into()
                .map_err(|_| Error::DbToRuntimeConversion)?,
            blinded_secret: PublicKey::from_slice(y).map_err(|_| Error::DbToRuntimeConversion)?,
            blind_signature: PublicKey::from_slice(c).map_err(|_| Error::DbToRuntimeConversion)?,
        });
    }

    Ok(ret)
}
