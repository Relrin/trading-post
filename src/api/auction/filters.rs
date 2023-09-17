use crate::core::orm::filter::Operator::{Gte, LikeContains, Lte};
use crate::core::orm::filter::{CustomFilter, Filter, IntoCustomFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
                let instance = CustomFilter::new(vec![Filter::new(
                    "item_name",
                    LikeContains(search_pattern),
                    None,
                )]);
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
}

impl<'a> IntoCustomFilter<'a> for ItemBidPriceRangeFilter<'a> {
    fn into_custom_filter(self) -> Option<CustomFilter<'a>> {
        let mut filters = vec![];

        match &self.params.min_price {
            Some(min_price) => {
                filters.push(Filter::new(
                    "bid_price",
                    Gte,
                    Some(min_price.to_owned().into()),
                ));
            }
            _ => {}
        }

        match &self.params.max_price {
            Some(max_price) => {
                filters.push(Filter::new(
                    "bid_price",
                    Lte,
                    Some(max_price.to_owned().into()),
                ));
            }
            _ => {}
        }

        match filters.len() > 0 {
            true => {
                let instance = CustomFilter::new(filters);
                Some(instance)
            }
            false => None,
        }
    }
}

pub struct ItemBuyoutPriceRangeFilter<'a> {
    params: &'a FilterParams,
}

impl<'a> ItemBuyoutPriceRangeFilter<'a> {
    pub fn new(params: &'a FilterParams) -> Self {
        Self { params }
    }
}

impl<'a> IntoCustomFilter<'a> for ItemBuyoutPriceRangeFilter<'a> {
    fn into_custom_filter(self) -> Option<CustomFilter<'a>> {
        let mut filters = vec![];

        match &self.params.min_buyout_price {
            Some(min_buyout_price) => {
                filters.push(Filter::new(
                    "buyout_price",
                    Gte,
                    Some(min_buyout_price.to_owned().into()),
                ));
            }
            _ => {}
        }

        match &self.params.max_buyout_price {
            Some(max_buyout_price) => {
                filters.push(Filter::new(
                    "buyout_price",
                    Lte,
                    Some(max_buyout_price.to_owned().into()),
                ));
            }
            _ => {}
        }

        match filters.len() > 0 {
            true => {
                let instance = CustomFilter::new(filters);
                Some(instance)
            }
            false => None,
        }
    }
}
