use cdrs_tokio::query::QueryValues;
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
}
