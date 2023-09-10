use cdrs_tokio::types::value::Value;

#[derive(Debug, Clone)]
pub struct Filter<'a> {
    field_name: &'a str,
    operator: Operator,
    value: Option<Value>,
}

impl<'a> Filter<'a> {
    pub fn new(field_name: &'a str, operator: Operator, value: Option<Value>) -> Self {
        Self {
            field_name,
            operator,
            value,
        }
    }

    pub fn get_field_name(&self) -> &'a str {
        self.field_name
    }

    pub fn get_operator(&self) -> Operator {
        self.operator.clone()
    }

    pub fn get_value(&self) -> Option<Value> {
        self.value.clone()
    }
}

#[derive(Debug, Clone)]
pub enum Operator {
    Eq,
    Lte,
    Gte,
    LikeContains(String),
}

impl Operator {
    pub fn to_string(&self) -> String {
        match self {
            Operator::Eq => String::from("="),
            Operator::Lte => String::from("<="),
            Operator::Gte => String::from(">="),
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
}

impl<'a> CustomFilter<'a> {
    pub fn new(filters: Vec<Filter<'a>>) -> Self {
        Self { filters }
    }

    pub fn get_filters(&self) -> &Vec<Filter<'a>> {
        &self.filters
    }
}
