use cdrs_tokio::query::QueryValues;
use serde::de::Unexpected::Str;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Filter<'a> {
    field_name: &'a str,
    operator: Operator,
}

impl<'a> Filter<'a> {
    pub fn new(field_name: &'a str, operator: Operator) -> Self {
        Self {
            field_name,
            operator,
        }
    }

    pub fn get_field_name(&self) -> &'a str {
        self.field_name
    }

    pub fn get_operator(&self) -> Operator {
        self.operator.clone()
    }
}

#[derive(Debug, Clone)]
pub enum Operator {
    Eq,
    LikeContains(String),
}

impl Operator {
    pub fn to_string(&self) -> String {
        match self {
            Operator::Eq => String::from("="),
            Operator::LikeContains(pattern) => pattern.to_owned(),
        }
    }
}

pub trait IntoCustomFilter<'a> {
    fn into_custom_filter(self) -> Option<CustomFilter<'a>>;
}

#[derive(Debug, Clone)]
pub struct CustomFilter<'a> {
    filters: Vec<Filter<'a>>,
    query_values: QueryValues,
}

impl<'a> CustomFilter<'a> {
    pub fn new(filters: Vec<Filter<'a>>, query_values: QueryValues) -> Self {
        Self {
            filters,
            query_values,
        }
    }

    pub fn get_filters(&self) -> &Vec<Filter<'a>> {
        &self.filters
    }

    pub fn get_query_values(&self) -> &QueryValues {
        &self.query_values
    }
}
