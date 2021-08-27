use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgConnection};
use url::Url;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub(crate) struct NewLink {
    destination: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, FromRow, Serialize)]
pub(crate) struct Link {
    id: Uuid,
    hash: String,
    pub(crate) destination: String,
}

#[derive(Debug, thiserror::Error, Serialize)]
pub(crate) enum NewLinkError {
    #[error("malformed url")]
    InvalidUrl,
    #[error("could not insert into database")]
    DatabaseError,
}

impl TryFrom<NewLink> for Link {
    type Error = NewLinkError;

    fn try_from(link: NewLink) -> Result<Self, Self::Error> {
        let dest = Url::parse(&link.destination).map_err(|_| NewLinkError::InvalidUrl)?;
        Ok(Self::new(&dest))
    }
}

impl Link {
    pub(crate) fn new(destination: &Url) -> Self {
        let id = Uuid::new_v4();
        let mut hash = base_x::encode(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
            &id.as_bytes()[..],
        );
        let _remainder = hash.split_off(5);

        Self {
            id,
            hash,
            destination: destination.to_string(),
        }
    }

    pub(crate) async fn insert(conn: &mut PgConnection, link: Link) -> Result<Self, NewLinkError> {
        sqlx::query_as!(
            Self,
            r#"INSERT INTO links (id, destination, hash)
            VALUES ($1, $2, $3)
            RETURNING id, destination, hash
            "#,
            link.id,
            link.destination,
            link.hash
        )
        .fetch_one(conn)
        .await
        .map_err(|_| NewLinkError::DatabaseError)
    }

    pub(crate) async fn get_by_hash(
        conn: &mut PgConnection,
        hash: &str,
    ) -> sqlx::Result<Option<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, destination, hash FROM links WHERE hash = $1",
            hash
        )
        .fetch_optional(conn)
        .await
    }

    pub(crate) async fn list(conn: &mut PgConnection) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            Self,
            "SELECT id, destination, hash FROM links ORDER BY destination"
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
