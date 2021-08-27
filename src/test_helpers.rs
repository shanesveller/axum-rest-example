use crate::{config::AppConfig, db};
use secrecy::Secret;
use sqlx::PgPool;

pub(crate) async fn test_db() -> anyhow::Result<PgPool> {
    let mut config = AppConfig::new()?;
    config.database.url =
        Secret::from(std::env::var("TEST_DATABASE_URL").expect("could not read TEST_DATABASE_URL"));

    Ok(db::new_pool(&config).await?)
}
