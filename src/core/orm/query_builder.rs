use crate::core::orm::filter::{CustomFilter, Filter, Operator};
use crate::core::orm::query::Query;
use std::collections::HashMap;

use cdrs_tokio::query::QueryValues;

#[derive(Clone)]
pub struct QueryBuilder<'a> {
    query_type: QueryType,
    table: &'a str,
    columns: &'a [&'a str],
    limit: Option<usize>,
    filters: Vec<Filter<'a>>,
    allow_filtering: bool,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(table: &'a str) -> Self {
        Self {
            query_type: QueryType::Select,
            table,
            columns: &[],
            limit: None,
            filters: vec![],
            allow_filtering: false,
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

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn filter_by(mut self, filter: Filter<'a>) -> Self {
        self.filters.insert(0, filter);
        self
    }

    pub fn custom_filters(mut self, custom_filters: &'a [&'a CustomFilter<'a>]) -> Self {
        for custom_filter in custom_filters {
            let filters = custom_filter.get_filters().clone();

            self.filters.extend(filters);
        }

        self
    }

    pub fn allow_filtering(mut self, value: bool) -> Self {
        self.allow_filtering = value;
        self
    }

    pub fn build(&self) -> Query {
        let raw_cql = match self.query_type {
            QueryType::Select => self.build_select_query(),
            QueryType::Insert => self.build_insert_query(),
            QueryType::Update => self.build_update_query(),
        };
        let query_values = self.get_query_values();

        Query::new(&raw_cql, query_values)
    }

    fn build_select_query(&self) -> String {
        let mut query = Vec::<String>::new();
        query.push(QueryType::Select.to_string());
        query.push(self.columns.join(", "));
        query.push("FROM".to_owned());
        query.push(self.table.to_owned());

        if self.filters.len() > 0 {
            query.push(self.build_where_clause());
        }

        if let Some(limit) = self.limit {
            query.push(format!("LIMIT {}", limit));
        }

        if self.allow_filtering {
            query.push("ALLOW FILTERING".to_owned());
        }

        query.join(" ")
    }

    fn build_insert_query(&self) -> String {
        let mut query = Vec::<String>::new();
        query.push(QueryType::Insert.to_string());
        query.push(self.table.to_owned());
        query.push(format!("({})", self.columns.join(", ")));
        query.push("VALUES".to_owned());
        query.push(format!(
            "({})",
            self.columns
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ")
        ));

        if self.allow_filtering {
            query.push("ALLOW FILTERING".to_owned());
        }

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
                .join(", "),
        );

        if self.filters.len() > 0 {
            query.push(self.build_where_clause());
        }

        if self.allow_filtering {
            query.push("ALLOW FILTERING".to_owned());
        }

        query.join(" ")
    }

    fn build_where_clause(&self) -> String {
        let conditions = self
            .filters
            .iter()
            .map(|filter| {
                let filter_operator = filter.get_operator();

                match filter_operator {
                    Operator::LikeContains(pattern) => {
                        format!("{} LIKE '{}'", filter.get_field_name(), pattern)
                    }
                    _ => format!(
                        "{} {} ?",
                        filter.get_field_name(),
                        filter_operator.to_string()
                    ),
                }
            })
            .collect::<Vec<String>>()
            .join(" AND ");

        format!("WHERE {}", conditions)
    }

    fn get_query_values(&self) -> QueryValues {
        let mut values = HashMap::new();

        for filter in self.filters.iter() {
            if let Some(value) = filter.get_value() {
                let field_name = filter.get_field_name().to_owned();
                values.insert(field_name, value);
            }
        }

        QueryValues::NamedValues(values)
    }
}

#[derive(Debug, Clone)]
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
            QueryType::Update => String::from("UPDATE"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::orm::filter::{CustomFilter, Filter, Operator};
    use crate::core::orm::query_builder::{QueryBuilder, QueryType};

    #[test]
    fn test_build_select_query() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade"
        );
    }

    #[test]
    fn test_build_select_query_with_limit() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .limit(1)
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade LIMIT 1"
        );
    }

    #[test]
    fn test_build_select_query_with_filter() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .filter_by(Filter::new("id", Operator::Eq, Some(5.into())))
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE id = ?"
        );
    }

    #[test]
    fn test_build_select_query_with_filter_and_limit() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .filter_by(Filter::new("id", Operator::Eq, Some(5.into())))
            .limit(1)
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE id = ? LIMIT 1"
        );
    }

    #[test]
    fn test_build_select_query_with_filter_and_allow_filtering() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .allow_filtering(true)
            .filter_by(Filter::new("id", Operator::Eq, Some(5.into())))
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE id = ? ALLOW FILTERING"
        );
    }

    #[test]
    fn test_build_select_query_with_filter_and_limit_and_allow_filtering() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .limit(1)
            .allow_filtering(true)
            .filter_by(Filter::new("id", Operator::Eq, Some(5.into())))
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE id = ? LIMIT 1 ALLOW FILTERING"
        );
    }

    #[test]
    fn test_build_select_query_with_custom_filters() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .custom_filters(&[&CustomFilter::new(vec![Filter::new(
                "item_id",
                Operator::Eq,
                Some(5.into()),
            )])])
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE item_id = ?"
        );
    }

    #[test]
    fn test_build_select_query_with_custom_filters_and_allow_filtering() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .allow_filtering(true)
            .custom_filters(&[&CustomFilter::new(vec![Filter::new(
                "item_id",
                Operator::Eq,
                Some(5.into()),
            )])])
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE item_id = ? ALLOW FILTERING"
        );
    }

    #[test]
    fn test_build_select_query_with_like_check() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .filter_by(Filter::new(
                "item_name",
                Operator::LikeContains(String::from("%sword")),
                None,
            ))
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE item_name LIKE '%sword'"
        );
    }

    #[test]
    fn test_build_select_query_with_filter_and_custom_filters_and_allow_filtering() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .allow_filtering(true)
            .filter_by(Filter::new("id", Operator::Eq, Some(5.into())))
            .custom_filters(&[&CustomFilter::new(vec![Filter::new(
                "item_id",
                Operator::Eq,
                Some(5.into()),
            )])])
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE id = ? AND item_id = ? ALLOW FILTERING"
        );
    }

    #[test]
    fn test_build_insert_query() {
        let query = QueryBuilder::new("trading_post.trade")
            .query_type(QueryType::Insert)
            .columns(&["key", "value"])
            .build_insert_query();

        assert_eq!(
            query,
            "INSERT INTO trading_post.trade (key, value) VALUES (?, ?)"
        );
    }

    #[test]
    fn test_build_update_query() {
        let query = QueryBuilder::new("trading_post.trade")
            .query_type(QueryType::Update)
            .columns(&["key", "value"])
            .build_update_query();

        assert_eq!(query, "UPDATE trading_post.trade SET key = ?, value = ?");
    }

    #[test]
    fn test_build_update_query_with_filters() {
        let query = QueryBuilder::new("trading_post.trade")
            .query_type(QueryType::Update)
            .columns(&["key", "value"])
            .filter_by(Filter::new("key", Operator::Eq, Some(5.into())))
            .build_update_query();

        assert_eq!(
            query,
            "UPDATE trading_post.trade SET key = ?, value = ? WHERE key = ?"
        );
    }

    #[test]
    fn test_build_update_query_with_filters_and_allow_filtering() {
        let query = QueryBuilder::new("trading_post.trade")
            .query_type(QueryType::Update)
            .columns(&["key", "value"])
            .allow_filtering(true)
            .filter_by(Filter::new("key", Operator::Eq, Some(5.into())))
            .build_update_query();

        assert_eq!(
            query,
            "UPDATE trading_post.trade SET key = ?, value = ? WHERE key = ? ALLOW FILTERING"
        );
    }

    #[test]
    fn test_build_update_query_with_custom_filters() {
        let query = QueryBuilder::new("trading_post.trade")
            .query_type(QueryType::Update)
            .columns(&["key", "value"])
            .custom_filters(&[&CustomFilter::new(vec![Filter::new(
                "key",
                Operator::Eq,
                Some(5.into()),
            )])])
            .build_update_query();

        assert_eq!(
            query,
            "UPDATE trading_post.trade SET key = ?, value = ? WHERE key = ?"
        );
    }

    #[test]
    fn test_build_update_query_with_custom_filters_and_allow_filtering() {
        let query = QueryBuilder::new("trading_post.trade")
            .query_type(QueryType::Update)
            .columns(&["key", "value"])
            .allow_filtering(true)
            .custom_filters(&[&CustomFilter::new(vec![Filter::new(
                "key",
                Operator::Eq,
                Some(5.into()),
            )])])
            .build_update_query();

        assert_eq!(
            query,
            "UPDATE trading_post.trade SET key = ?, value = ? WHERE key = ? ALLOW FILTERING"
        );
    }

    #[test]
    fn test_build_update_query_with_filters_and_custom_filters_and_allow_filtering() {
        let query = QueryBuilder::new("trading_post.trade")
            .query_type(QueryType::Update)
            .columns(&["key", "value"])
            .allow_filtering(true)
            .filter_by(Filter::new("key", Operator::Eq, Some(5.into())))
            .custom_filters(&[&CustomFilter::new(vec![Filter::new(
                "value",
                Operator::Eq,
                Some(5.into()),
            )])])
            .build_update_query();

        assert_eq!(
            query,
            "UPDATE trading_post.trade SET key = ?, value = ? WHERE key = ? AND value = ? ALLOW FILTERING"
        );
    }

    #[test]
    fn test_custom_filters_appear_always_in_end() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .allow_filtering(true)
            // intentionally placed before for builder behaviour check
            .custom_filters(&[&CustomFilter::new(vec![Filter::new(
                "item_id",
                Operator::Eq,
                Some(5.into()),
            )])])
            .filter_by(Filter::new("id", Operator::Eq, Some(5.into())))
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE id = ? AND item_id = ? ALLOW FILTERING"
        );
    }
}
