pub mod session;
use redis::aio::ConnectionManager;
use uuid::Uuid;

pub async fn init_redis(redis_url: &str) -> Result<ConnectionManager, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    let conn = client.get_connection_manager().await?;
    Ok(conn)
}

pub fn session_key(token_hash: &str) -> String {
    format!("session:{token_hash}")
}

pub fn user_key(user_id: Uuid) -> String {
    format!("user_sessions:{user_id}")
}
