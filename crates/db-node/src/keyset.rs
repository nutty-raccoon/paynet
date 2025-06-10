use std::str::FromStr;

use nuts::nut02::KeysetId;
use sqlx::PgConnection;

use crate::Error;

#[derive(Debug, Clone)]
pub struct KeysetInfo<U> {
    unit: U,
    active: bool,
    max_order: u8,
    derivation_path_index: u32,
}

impl<U> KeysetInfo<U> {
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn max_order(&self) -> u8 {
        self.max_order
    }
    pub fn derivation_path_index(&self) -> u32 {
        self.derivation_path_index
    }
}

impl<U: Clone> KeysetInfo<U> {
    pub fn unit(&self) -> U {
        self.unit.clone()
    }
}

pub async fn get_keysets(
    conn: &mut PgConnection,
) -> Result<impl Iterator<Item = ([u8; 8], String, bool)>, sqlx::Error> {
    let record = sqlx::query!("SELECT id, unit, active FROM keyset")
        .fetch_all(conn)
        .await?;

    Ok(record
        .into_iter()
        .map(|r| (r.id.to_be_bytes(), r.unit, r.active)))
}

pub async fn get_keyset<U: FromStr>(
    conn: &mut PgConnection,
    keyset_id: &KeysetId,
) -> Result<KeysetInfo<U>, Error> {
    let record = sqlx::query!(
        r#"SELECT unit, active, max_order, derivation_path_index
        FROM keyset
        WHERE id = $1"#,
        keyset_id.as_i64()
    )
    .fetch_one(conn)
    .await?;

    let info = KeysetInfo {
        unit: U::from_str(&record.unit).map_err(|_| Error::InvalidUnit(record.unit))?,
        active: record.active,
        max_order: u8::try_from(record.max_order).map_err(|_| Error::DbToRuntimeConversion)?,
        derivation_path_index: u32::from_be_bytes(record.derivation_path_index.to_be_bytes()),
    };

    Ok(info)
}

pub async fn get_active_keyset_for_unit(
    conn: &mut PgConnection,
    unit: String,
) -> Result<[u8; 8], sqlx::Error> {
    let record = sqlx::query!(
        r#"SELECT id
        FROM keyset
        WHERE unit = $1 AND active = true"#,
        unit
    )
    .fetch_one(conn)
    .await?;

    Ok(record.id.to_be_bytes())
}

pub async fn get_active_keysets<U: FromStr>(
    conn: &mut PgConnection,
) -> Result<Vec<(KeysetId, KeysetInfo<U>)>, Error> {
    let records = sqlx::query!(
        r#"SELECT id, unit, active, max_order, derivation_path_index
        FROM keyset
        WHERE active = TRUE"#,
    )
    .fetch_all(conn)
    .await?;

    let keysets_info = records
        .into_iter()
        .map(|record| -> Result<(_, KeysetInfo<U>), Error> {
            Ok((
                KeysetId::from_bytes(&record.id.to_be_bytes())
                    .map_err(|_| Error::DbToRuntimeConversion)?,
                KeysetInfo {
                    unit: U::from_str(&record.unit).map_err(|_| Error::InvalidUnit(record.unit))?,
                    active: record.active,
                    max_order: u8::try_from(record.max_order)
                        .map_err(|_| Error::DbToRuntimeConversion)?,
                    derivation_path_index: u32::from_be_bytes(
                        record.derivation_path_index.to_be_bytes(),
                    ),
                },
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(keysets_info)
}

pub async fn deactivate_keysets(conn: &mut PgConnection, keyset_ids: &[i64]) -> Result<(), Error> {
    sqlx::query!(
        "UPDATE keyset SET active = false WHERE id = ANY($1)",
        keyset_ids
    )
    .execute(conn)
    .await?;

    Ok(())
}
