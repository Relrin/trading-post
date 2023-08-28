use crate::core::orm::filter::Operator::LikeContains;
use crate::core::orm::filter::{CustomFilter, Filter, IntoCustomFilter};
use cdrs_tokio::query::QueryValues;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize, PartialEq)]
pub struct FilterParams {
    pub(crate) name: Option<String>,
    pub(crate) min_price: Option<i64>,
    pub(crate) max_price: Option<i64>,
    pub(crate) min_buyout_price: Option<i64>,
    pub(crate) max_buyout_price: Option<i64>,
}

pub struct ItemNameFilter<'a> {
    params: &'a FilterParams,
}

impl<'a> ItemNameFilter<'a> {
    pub fn new(params: &'a FilterParams) -> Self {
        Self { params }
    }
}

impl<'a> IntoCustomFilter<'a> for ItemNameFilter<'a> {
    fn into_custom_filter(self) -> Option<CustomFilter<'a>> {
        match &self.params.name {
            Some(name) => {
                let search_pattern = format!("%{0}%", name);
                let instance = CustomFilter::new(
                    vec![Filter::new("item_name", LikeContains(search_pattern))],
                    QueryValues::SimpleValues(vec![]),
                );
                Some(instance)
            }
            None => None,
        }
    }
}
