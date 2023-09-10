use cdrs_tokio::frame::{Envelope, TryFromRow};
use cdrs_tokio::query::{QueryParamsBuilder, QueryValues};
use cdrs_tokio::types::prelude::Value;
use cdrs_tokio::types::rows::Row;
use log::error;
use serde::Serialize;

use crate::core::error::{Error, Result};
use crate::core::orm::session::CassandraSession;
use crate::core::pagination::PaginationParams;

#[derive(Debug)]
pub struct Query {
    raw_cql: String,
    filter_values: QueryValues,
}

impl Query {
    pub fn new(raw_cql: &str, filter_values: QueryValues) -> Self {
        Query {
            raw_cql: raw_cql.to_owned(),
            filter_values,
        }
    }

    pub async fn insert(&self, session: &CassandraSession, query_values: &QueryValues) {
        session
            .query_with_values(&self.raw_cql, query_values.to_owned())
            .await
            .expect("Error inserting data");
    }

    pub async fn update(
        &self,
        session: &CassandraSession,
        query_values: &QueryValues,
    ) -> Result<Envelope> {
        session
            .query_with_values(&self.raw_cql, query_values.to_owned())
            .await
            .map(|env| env)
            .map_err(|err| {
                error!("{}", err);
                err.into()
            })
    }

    pub async fn get_instance<T>(
        &self,
        session: &CassandraSession,
        query_values: &QueryValues,
    ) -> Result<T>
    where
        T: Serialize + TryFromRow,
    {
        let all_query_values = self.get_merged_query_values(&query_values);

        let rows = session
            .query_with_values(&self.raw_cql, all_query_values)
            .await
            .map_err(|err| {
                error!("{}", err);
                Error::CassandraError {
                    message: String::from("Object was not found or doesn't exist."),
                }
            })
            .map(|envelope| envelope.response_body())
            .map_err(|err| {
                error!("{}", err);
                Error::CassandraError {
                    message: String::from("Can't read the response body."),
                }
            })
            .map(|response_body| response_body.unwrap().into_rows())?
            .unwrap_or(vec![]);

        if rows.len() == 0 {
            return Err(Error::CassandraError {
                message: String::from("Object was not found or doesn't exist."),
            });
        }

        let row = rows.first().unwrap().to_owned();
        Ok(T::try_from_row(row).expect("decode row"))
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
        let all_query_values = self.get_merged_query_values(&query_values);
        let mut pager = session.paged(pagination_params.page_size);
        let mut query_pager = pager.query_with_params(
            &self.raw_cql,
            QueryParamsBuilder::new()
                .with_values(all_query_values)
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

    fn get_merged_query_values(&self, query_values: &QueryValues) -> QueryValues {
        let mut values = Vec::<Value>::new();
        self.copy_query_values_into_vec(&mut values, query_values);
        self.copy_query_values_into_vec(&mut values, &self.filter_values);
        QueryValues::SimpleValues(values)
    }

    fn copy_query_values_into_vec(&self, container: &mut Vec<Value>, query_values: &QueryValues) {
        match &query_values {
            QueryValues::SimpleValues(vec) => container.extend_from_slice(&vec),
            QueryValues::NamedValues(hash_map) => {
                for value in hash_map.values() {
                    container.push(value.clone())
                }
            }
        };
    }
}
