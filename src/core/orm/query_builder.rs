// TODO: Add filters support
use crate::core::orm::query::Query;

#[derive(Debug)]
pub struct QueryBuilder<'a> {
    query_type: QueryType,
    table: &'a str,
    columns: &'a [&'a str],
}

impl<'a> QueryBuilder<'a> {
    pub fn new(table: &'a str) -> Self {
        QueryBuilder {
            query_type: QueryType::Select,
            table,
            columns: &[],
        }
    }

    pub fn query_type(mut self, query_type: QueryType) -> Self {
        self.query_type = query_type;
        self
    }

    pub fn columns(mut self, columns: &'a [&'a str]) -> Self {
        self.columns = columns;
        self
    }

    pub fn build(&self) -> Query {
        let raw_cql = match self.query_type {
            QueryType::Select => self.build_select_query(),
            QueryType::Insert => self.build_insert_query(),
            QueryType::Update => self.build_update_query(),
        };

        Query::new(&raw_cql)
    }

    fn build_select_query(&self) -> String {
        let mut query = Vec::<String>::new();
        query.push(QueryType::Select.to_string());
        query.push(self.columns.join(", "));
        query.push("FROM".to_owned());
        query.push(self.table.to_owned());

        query.join(" ")
    }

    fn build_insert_query(&self) -> String {
        let mut query = Vec::<String>::new();
        query.push(QueryType::Insert.to_string());
        query.push(self.table.to_owned());
        query.push( format!("({})", self.columns.join(", ")));
        query.push("VALUES".to_owned());
        query.push( format!("({})", self.columns.iter().map(|_| "?").collect::<Vec<_>>().join(", ")));

        query.join(" ")
    }

    fn build_update_query(&self) -> String {
        let mut query = Vec::<String>::new();
        query.push(QueryType::Update.to_string());
        query.push(self.table.to_owned());
        query.push("SET".to_owned());
        query.push(
            self.columns
                .iter()
                .map(|field_name| format!("{} = ?", field_name))
                .collect::<Vec<String>>()
                .join(", ")
        );

        query.join(" ")
    }
}

#[derive(Debug)]
pub enum QueryType {
    Select,
    Insert,
    Update,
}

impl QueryType {
    fn to_string(&self) -> String {
        match self {
            QueryType::Select => String::from("SELECT"),
            QueryType::Insert => String::from("INSERT INTO"),
            QueryType::Update => String::from("UPDATE")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::orm::query_builder::{QueryBuilder, QueryType};

    #[test]
    fn test_build_select_query() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .build_select_query();

        assert_eq!(query, "SELECT id, item_id, item_name FROM trading_post.trade");
    }

    #[test]
    fn test_build_insert_query() {
        let query = QueryBuilder::new("trading_post.trade")
            .query_type(QueryType::Insert)
            .columns(&["key", "value"])
            .build_insert_query();

        assert_eq!(query, "INSERT INTO trading_post.trade (key, value) VALUES (?, ?)");
    }

    #[test]
    fn test_build_update_query() {
        let query = QueryBuilder::new("trading_post.trade")
            .query_type(QueryType::Update)
            .columns(&["key", "value"])
            .build_update_query();

        assert_eq!(query, "UPDATE trading_post.trade SET key = ?, value = ?");
    }
}