// TODO: Add filters support
// TODO: Add Query type (that holds CQL, page size, page number) + execute pub method

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

    pub fn build(&self) -> String {
        match self.query_type {
            QueryType::Select => self.build_select_query(),
            QueryType::Insert => self.build_insert_query(),
            QueryType::Update => self.build_update_query(),
        }
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
        "".to_owned()
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
    fn test_select_query() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .build();

        assert_eq!(query, "SELECT id, item_id, item_name FROM trading_post.trade");
    }

    #[test]
    fn test_insert_query() {
        let query = QueryBuilder::new("trading_post.trade")
            .query_type(QueryType::Insert)
            .columns(&["key", "value"])
            .build();

        assert_eq!(query, "INSERT INTO trading_post.trade (key, value) VALUES (?, ?)");
    }
}