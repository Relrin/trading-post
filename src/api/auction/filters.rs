use crate::core::orm::filter::Operator::{Gte, LikeContains, Lte};
use crate::core::orm::filter::{CustomFilter, Filter, IntoCustomFilter};
use cdrs_tokio::query::QueryValues;
use cdrs_tokio::types::value::Value;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Validate, Serialize, Deserialize, PartialEq)]
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

pub struct ItemBidPriceRangeFilter<'a> {
    params: &'a FilterParams,
}

impl<'a> ItemBidPriceRangeFilter<'a> {
    pub fn new(params: &'a FilterParams) -> Self {
        Self { params }
    }

    fn get_min_price_filter(&self) -> Option<(Filter<'a>, i64)> {
        match &self.params.min_price {
            Some(min_price) => Some((Filter::new("bid_price", Gte), min_price.clone())),
            None => None,
        }
    }

    fn get_max_price_filter(&self) -> Option<(Filter<'a>, i64)> {
        match &self.params.max_price {
            Some(max_price) => Some((Filter::new("bid_price", Lte), max_price.clone())),
            None => None,
        }
    }
}

impl<'a> IntoCustomFilter<'a> for ItemBidPriceRangeFilter<'a> {
    fn into_custom_filter(self) -> Option<CustomFilter<'a>> {
        let mut filters = vec![];
        let mut values: Vec<Value> = vec![];

        if let Some((min_price_filter, min_price)) = self.get_min_price_filter() {
            filters.push(min_price_filter);
            values.push(min_price.into());
        }

        if let Some((max_price_filter, max_price)) = self.get_max_price_filter() {
            filters.push(max_price_filter);
            values.push(max_price.into());
        }

        match filters.len() > 0 {
            true => {
                let instance = CustomFilter::new(filters, QueryValues::SimpleValues(values));
                Some(instance)
            }
            false => None,
        }
    }
}
