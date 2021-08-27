use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgConnection};
use url::Url;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Deserialize, FromRow, Serialize)]
struct Link {
    id: Uuid,
    destination: String,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
impl Link {
    fn new(destination: &Url) -> Self {
        Self {
            id: Uuid::new_v4(),
            destination: destination.to_string(),
        }
    }

    async fn insert(conn: &mut PgConnection, link: Link) -> sqlx::Result<Self> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO links (id, destination)
            VALUES ($1, $2)
            RETURNING id, destination
            "#,
            link.id,
            link.destination
        )
        .fetch_one(conn)
        .await
    }

    async fn list(conn: &mut PgConnection) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, destination FROM links ORDER BY destination"
        )
        .fetch_all(conn)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_db;
    use anyhow::Result;

    #[test]
    fn test_new() -> Result<()> {
        let url = Url::parse("https://www.google.com")?;
        assert_eq!(url.to_string(), Link::new(&url).destination);
        Ok(())
    }

    #[tokio::test]
    async fn test_insert() -> Result<()> {
        let pool = test_db().await?;
        let mut conn = pool.begin().await?;

        let url = Url::parse("https://www.google.com")?;
        let link = Link::new(&url);
        let inserted = Link::insert(&mut conn, link.clone()).await?;

        assert_eq!(inserted, link);
        Ok(())
    }

    #[tokio::test]
    async fn test_list() -> Result<()> {
        let pool = test_db().await?;
        let mut conn = pool.begin().await?;

        let url = Url::parse("https://www.google.com")?;
        let link = Link::new(&url);
        let inserted = Link::insert(&mut conn, link).await?;

        let list = Link::list(&mut conn).await?;

        assert_eq!(list, vec![inserted]);
        Ok(())
    }
}
