use cdrs_tokio::frame::TryFromRow;
use cdrs_tokio::query::{QueryParamsBuilder, QueryValues};
use cdrs_tokio::types::rows::Row;
use serde::Serialize;

use crate::core::error::Result;
use crate::core::orm::session::CassandraSession;
use crate::core::pagination::PaginationParams;

#[derive(Debug)]
pub struct Query {
    raw_cql: String,
}

impl Query {
    pub fn new(raw_cql: &str) -> Self {
        Query {
            raw_cql: raw_cql.to_owned(),
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
        pagination_params: &PaginationParams,
    ) -> Result<Vec<T>>
    where
        T: Serialize + TryFromRow,
    {
        let mut pager = session.paged(pagination_params.page_size);
        let mut query_pager = pager.query_with_params(
            &self.raw_cql,
            QueryParamsBuilder::new()
                .with_values(query_values.to_owned())
                .build(),
        );

        let mut current_page = 1;
        let mut rows: Vec<Row>;
        loop {
            rows = query_pager.next().await?;

            if !query_pager.has_more() || current_page >= pagination_params.page {
                break;
            }

            current_page += 1;
        }

        if current_page < pagination_params.page {
            rows.clear();
        }

        Ok(rows
            .into_iter()
            .map(|row| T::try_from_row(row).expect("decode row"))
            .collect())
    }
}
