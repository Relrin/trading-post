
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
}
