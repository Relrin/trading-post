use std::collections::HashMap;

use cdrs_tokio::frame::{Envelope, TryFromRow};
use cdrs_tokio::query::{QueryParamsBuilder, QueryValues};
use cdrs_tokio::types::rows::Row;
use log::error;
use serde::Serialize;

use crate::core::error::{Error, Result};
use crate::core::orm::session::CassandraSession;
use crate::core::pagination::PaginationParams;

#[derive(Debug)]
pub struct Query {
    raw_cql: String,
    query_values: QueryValues,
}

impl Query {
    pub fn new(raw_cql: &str, query_values: QueryValues) -> Self {
        Query {
            raw_cql: raw_cql.to_owned(),
            query_values,
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
        let update_query_values = self.get_merged_query_values(&query_values);

        session
            .query_with_values(&self.raw_cql, update_query_values)
            .await
            .map(|env| env)
            .map_err(|err| {
                error!("{}", err);
                err.into()
            })
    }

    pub async fn get_instance<T>(&self, session: &CassandraSession) -> Result<T>
    where
        T: Serialize + TryFromRow,
    {
        let rows = session
            .query_with_values(&self.raw_cql, self.query_values.to_owned())
            .await
            .map_err(|err| {
                error!("{}", err);
                Error::CassandraError("Object was not found or doesn't exist.".to_string())
            })
            .map(|envelope| envelope.response_body())
            .map_err(|err| {
                error!("{}", err);
                Error::CassandraError("Can't read the response body.".to_string())
            })
            .map(|response_body| response_body.unwrap().into_rows())?
            .unwrap_or(vec![]);

        if rows.len() == 0 {
            return Err(Error::CassandraError(
                "Object was not found or doesn't exist.".to_string(),
            ));
        }

        let row = rows.first().unwrap().to_owned();
        Ok(T::try_from_row(row).expect("decode row"))
    }

    pub async fn get_paginated_entries<T>(
        &self,
        session: &CassandraSession,
        pagination_params: &PaginationParams,
    ) -> Result<Vec<T>>
    where
        T: Serialize + TryFromRow,
    {
        let mut pager = session.paged(pagination_params.page_size);
        let mut query_pager = pager.query_with_params(
            &self.raw_cql,
            QueryParamsBuilder::new()
                .with_values(self.query_values.to_owned())
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

    fn get_merged_query_values(&self, custom_query_values: &QueryValues) -> QueryValues {
        match custom_query_values {
            QueryValues::SimpleValues(_) => self.get_merged_simple_values(custom_query_values),
            QueryValues::NamedValues(_) => self.get_merged_named_values(custom_query_values),
        }
    }

    fn get_merged_simple_values(&self, custom_query_values: &QueryValues) -> QueryValues {
        let mut values = Vec::new();

        match &self.query_values {
            QueryValues::SimpleValues(vec) => values.extend_from_slice(vec),
            _ => {}
        }

        match &custom_query_values {
            QueryValues::SimpleValues(vec) => values.extend_from_slice(vec),
            _ => {}
        }

        QueryValues::SimpleValues(values)
    }

    fn get_merged_named_values(&self, custom_query_values: &QueryValues) -> QueryValues {
        let mut values = HashMap::new();

        match &self.query_values {
            QueryValues::NamedValues(hm) => {
                values.extend(hm.into_iter().map(|(k, v)| (k.clone(), v.clone())))
            }
            _ => {}
        }

        match &custom_query_values {
            QueryValues::NamedValues(hm) => {
                values.extend(hm.into_iter().map(|(k, v)| (k.clone(), v.clone())))
            }
            _ => {}
        }

        QueryValues::NamedValues(values)
    }
}
