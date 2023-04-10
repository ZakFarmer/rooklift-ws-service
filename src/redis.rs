use redis::{aio::Connection, AsyncCommands, RedisError};

// Retrieve a Redis connection from the pool
pub async fn get_con(client: redis::Client) -> Result<Connection, RedisError> {
    client.get_async_connection().await
}

// Set a string value in Redis
pub async fn set_str(
    con: &mut Connection,
    key: &str,
    value: &str,
    ttl_seconds: usize,
) -> Result<(), RedisError> {
    con.set(key, value).await.map_err(RedisError::from)?;

    if ttl_seconds > 0 {
        con.expire(key, ttl_seconds)
            .await
            .map_err(RedisError::from)?;
    }

    Ok(())
}
