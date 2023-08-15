use cdrs_tokio::frame::TryFromRow;
use cdrs_tokio::query::QueryValues;
use serde::Serialize;

use crate::core::error::Result;
use crate::core::orm::session::CassandraSession;

#[derive(Debug)]
pub struct Query {
    raw_cql: String,
    page_size: usize,
    page_number: usize,
}

impl Query {
    pub fn new(raw_cql: &str) -> Self {
        Query {
            raw_cql: raw_cql.to_owned(),
            page_size: 0,
            page_number: 0,
        }
    }

    pub async fn insert(&self, session: &CassandraSession, query_values: &QueryValues) {
        session
            .query_with_values(&self.raw_cql, query_values.to_owned())
            .await
            .expect("Error inserting data");
    }

    pub async fn get_paginated_entries<T>(
        &self,
        session: &CassandraSession,
        query_values: &QueryValues,
    ) -> Result<Vec<T>>
    where
        T: Serialize + TryFromRow,
    {
        let rows = session
            .query_with_values(&self.raw_cql, query_values.to_owned())
            .await?;

        Ok(rows
            .response_body()
            .expect("get body")
            .into_rows()
            .expect("transform into rows")
            .into_iter()
            .map(|row| T::try_from_row(row).expect("decode row"))
            .collect())
    }
}
