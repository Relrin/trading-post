use crate::core::orm::filter::{CustomFilter, Filter, Operator};
use crate::core::orm::query::Query;
use cdrs_tokio::query::QueryValues;
use cdrs_tokio::types::value::Value;

#[derive(Clone)]
pub struct QueryBuilder<'a> {
    query_type: QueryType,
    table: &'a str,
    columns: &'a [&'a str],
    filters: Vec<Filter<'a>>,
    filter_values: Vec<QueryValues>,
    allow_filtering: bool,
}

impl<'a> QueryBuilder<'a> {
    pub fn new(table: &'a str) -> Self {
        Self {
            query_type: QueryType::Select,
            table,
            columns: &[],
            filters: vec![],
            filter_values: vec![],
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

    pub fn filter_by(mut self, filter: Filter<'a>) -> Self {
        self.filters.insert(0, filter);
        self
    }

    pub fn custom_filters(mut self, custom_filters: &'a [&'a CustomFilter<'a>]) -> Self {
        for custom_filter in custom_filters {
            let filters = custom_filter.get_filters().clone();
            let query_values = custom_filter.get_query_values().clone();

            self.filters.extend(filters);
            self.filter_values.push(query_values);
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
        let filter_values = self.merge_filter_values();

        Query::new(&raw_cql, filter_values)
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
                        format!("{} LIKE {}", filter.get_field_name(), pattern)
                    }
                    _ => format!(
                        "{} {} ?",
                        filter.get_field_name(),
                        filter_operator.to_string()
                    ),
                }
            })
            .collect::<Vec<String>>()
            .join(", ");

        match self.allow_filtering {
            true => format!("WHERE {} ALLOW FILTERING", conditions),
            false => format!("WHERE {}", conditions),
        }
    }

    fn merge_filter_values(&self) -> QueryValues {
        let mut values = Vec::<Value>::new();

        for filter_values in self.filter_values.iter() {
            match filter_values {
                QueryValues::SimpleValues(vec) => {
                    for value in vec {
                        values.push(value.clone())
                    }
                }
                QueryValues::NamedValues(hash_map) => {
                    for value in hash_map.values() {
                        values.push(value.clone())
                    }
                }
            }
        }

        QueryValues::SimpleValues(values)
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
    use crate::core::orm::query::Query;
    use crate::core::orm::query_builder::{QueryBuilder, QueryType};
    use cdrs_tokio::query::QueryValues;
    use cdrs_tokio::query_values;
    use cdrs_tokio::types::value::Value;

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
    fn test_build_select_query_with_filter() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .filter_by(Filter::new("id", Operator::Eq))
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE id = ?"
        );
    }

    #[test]
    fn test_build_select_query_with_filter_and_allow_filtering() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .allow_filtering(true)
            .filter_by(Filter::new("id", Operator::Eq))
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE id = ? ALLOW FILTERING"
        );
    }

    #[test]
    fn test_build_select_query_with_custom_filters() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .custom_filters(&[&CustomFilter::new(
                vec![Filter::new("item_id", Operator::Eq)],
                query_values!(5),
            )])
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
            .custom_filters(&[&CustomFilter::new(
                vec![Filter::new("item_id", Operator::Eq)],
                query_values!(5),
            )])
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE item_id = ? ALLOW FILTERING"
        );
    }

    #[test]
    fn test_build_select_query_with_filter_and_custom_filters_and_allow_filtering() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .allow_filtering(true)
            .filter_by(Filter::new("id", Operator::Eq))
            .custom_filters(&[&CustomFilter::new(
                vec![Filter::new("item_id", Operator::Eq)],
                query_values!(5),
            )])
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE id = ?, item_id = ? ALLOW FILTERING"
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
            .filter_by(Filter::new("key", Operator::Eq))
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
            .filter_by(Filter::new("key", Operator::Eq))
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
            .custom_filters(&[&CustomFilter::new(
                vec![Filter::new("key", Operator::Eq)],
                query_values!(5),
            )])
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
            .custom_filters(&[&CustomFilter::new(
                vec![Filter::new("key", Operator::Eq)],
                query_values!(5),
            )])
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
            .filter_by(Filter::new("key", Operator::Eq))
            .custom_filters(&[&CustomFilter::new(
                vec![Filter::new("value", Operator::Eq)],
                query_values!(5),
            )])
            .build_update_query();

        assert_eq!(
            query,
            "UPDATE trading_post.trade SET key = ?, value = ? WHERE key = ?, value = ? ALLOW FILTERING"
        );
    }

    #[test]
    fn test_custom_filters_appear_always_in_end() {
        let query = QueryBuilder::new("trading_post.trade")
            .columns(&["id", "item_id", "item_name"])
            .allow_filtering(true)
            // intentionally placed before for builder behaviour check
            .custom_filters(&[&CustomFilter::new(
                vec![Filter::new("item_id", Operator::Eq)],
                query_values!(5),
            )])
            .filter_by(Filter::new("id", Operator::Eq))
            .build_select_query();

        assert_eq!(
            query,
            "SELECT id, item_id, item_name FROM trading_post.trade WHERE id = ?, item_id = ? ALLOW FILTERING"
        );
    }

    #[test]
    fn test_merge_filter_values() {
        let filter_key =
            &CustomFilter::new(vec![Filter::new("key", Operator::Eq)], query_values!(5));
        let filter_value = &CustomFilter::new(
            vec![Filter::new("value", Operator::Eq)],
            query_values!("key" => "abc"),
        );
        let custom_filters = [filter_key, filter_value];
        let query = QueryBuilder::new("").custom_filters(&custom_filters);

        assert_eq!(
            query.merge_filter_values(),
            QueryValues::SimpleValues(vec![
                Value::Some(vec![0, 0, 0, 5]), // 5
                Value::Some(vec![97, 98, 99]), // abc
            ])
        );
    }
}
