use nuts::{
    Amount,
    nut00::Proof,
    nut05::{MeltMethodSettings, MeltQuoteResponse, MeltQuoteState},
};
use sqlx::{PgConnection, PgPool};
use starknet_types::{MeltPaymentRequest, Unit};
use uuid::Uuid;

use crate::{
    app_state::SignerClient, keyset_cache::KeysetCache, logic::process_melt_inputs,
    methods::Method, utils::unix_time,
};

use super::errors::Error;

pub async fn validate_and_register_quote(
    conn: &mut PgConnection,
    signer: SignerClient,
    keyset_cache: KeysetCache,
    settings: MeltMethodSettings<Method, Unit>,
    mint_ttl: u64,
    melt_payment_request: MeltPaymentRequest,
    inputs: &[Proof],
) -> Result<(Uuid, Amount, Amount, u64), Error> {
    if !settings.unit.is_asset_supported(melt_payment_request.asset) {
        return Err(Error::InvalidAssetForUnit(
            melt_payment_request.asset,
            settings.unit,
        ));
    }

    let mut tx = db_node::start_db_tx_from_conn(conn)
        .await
        .map_err(Error::TxBegin)?;

    let (total_amount, insert_spent_proof_query) =
        process_melt_inputs(&mut tx, signer, keyset_cache, inputs).await?;

    if let Some(min_amount) = settings.min_amount {
        if min_amount > total_amount {
            Err(Error::AmountTooLow(total_amount, min_amount))?;
        }
    }
    if let Some(max_amount) = settings.max_amount {
        if max_amount < total_amount {
            Err(Error::AmountTooHigh(max_amount, total_amount))?;
        }
    }

    let expiry = unix_time() + mint_ttl;
    let quote = Uuid::new_v4();
    // Arbitrary for now, but will be enough to pay tx fee on starknet
    let fee = Amount::ONE;

    db_node::melt_quote::insert_new(
        &mut tx,
        quote,
        settings.unit,
        total_amount,
        fee,
        &serde_json::to_string(&melt_payment_request)
            .expect("it has been deserialized it should be serializable"),
        expiry,
    )
    .await?;
    insert_spent_proof_query.execute(&mut tx).await?;
    tx.commit().await?;

    Ok((quote, total_amount, fee, expiry))
}

// pub async fn handle_unpaid_quote(
//     conn: &mut PgConnection,
//     signer: SharedSignerClient,
//     keyset_cache: KeysetCache,
//     quote_id: Uuid,
//     inputs: &[Proof],
// ) -> Result<(), Error> {
//     let mut tx = db_node::start_db_tx_from_conn(conn)
//         .await
//         .map_err(Error::TxBegin)?;

//     // let (quote_unit, expected_amount, fee_reserve, mut state, expiry) =
//     //     db_node::melt_quote::get_data::<Unit>(&mut tx, quote_id).await?;

//     let (total_amount, insert_spent_proofs_query_builder) =
//         process_melt_inputs(&mut tx, signer, keyset_cache, &inputs).await?;

//     // All verifications done, melt the tokens, and update state
//     insert_spent_proofs_query_builder.execute(&mut tx).await?;
//     db_node::melt_quote::set_state(&mut tx, quote_id, MeltQuoteState::Pending).await?;
//     tx.commit().await?;

//     Ok(())
// }

pub async fn starknet_melt(
    pool: PgPool,
    signer: SignerClient,
    keyset_cache: KeysetCache,
    settings: MeltMethodSettings<Method, Unit>,
    melt_ttl: u64,
    melt_payment_request: MeltPaymentRequest,
    inputs: &[Proof],
) -> Result<MeltQuoteResponse<Uuid>, Error> {
    let mut conn = pool.acquire().await?;

    let (quote_id, total_amount, fee, expiry) = validate_and_register_quote(
        &mut conn,
        signer,
        keyset_cache.clone(),
        settings,
        melt_ttl,
        melt_payment_request,
        inputs,
    )
    .await?;

    let state = proceed_to_payment(&mut conn, quote_id).await?;

    Ok(MeltQuoteResponse {
        quote: quote_id,
        amount: total_amount,
        fee,
        state,
        expiry,
    })
}

#[cfg(feature = "uncollateralized")]
async fn proceed_to_payment(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<MeltQuoteState, Error> {
    let new_state = MeltQuoteState::Paid;

    db_node::melt_quote::set_state(conn, quote_id, new_state).await?;
    Ok(new_state)
}

#[cfg(not(feature = "uncollateralized"))]
async fn proceed_to_payment(
    conn: &mut PgConnection,
    quote_id: Uuid,
) -> Result<MeltQuoteState, Error> {
    // TODO: actually proceed to payment

    let new_state = MeltQuoteState::Pending;

    db_node::melt_quote::set_state(conn, quote_id, new_state).await?;
    Ok(new_state)
}
