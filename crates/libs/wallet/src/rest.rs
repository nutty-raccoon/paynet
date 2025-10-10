pub async fn register(
    pool: Pool<SqliteConnectionManager>,
    node_url: &NodeUrl,
) -> Result<(NodeClient<tonic::transport::Channel>, u32), RegisterNodeError> {
    let mut node_client = NodeClient::connect(node_url.to_string()).await?;

    let node_id = {
        let db_conn = pool.get()?;
        db::node::insert(&db_conn, node_url)?;
        db::node::get_id_by_url(&db_conn, node_url)?
            .ok_or(RegisterNodeError::NotFound(node_url.clone()))?
    };

    refresh_keysets(pool, &mut node_client, node_id)
        .await
        .map_err(|e| RegisterNodeError::RefreshNodeKeyset(node_id, e))?;

    Ok((node_client, node_id))
}
