
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
}

impl Operator {
    pub fn to_string(&self) -> String {
        match self {
            Operator::Eq => String::from("="),
        }
    }
}