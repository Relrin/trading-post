use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Validate, Deserialize, PartialEq)]
pub struct PaginationParams {
    #[validate(range(min = 1))]
    #[serde(default = "get_default_page")]
    pub(crate) page: i32,
    #[validate(range(min = 10))]
    #[serde(default = "get_default_page_size")]
    pub(crate) page_size: i32,
}

fn get_default_page() -> i32 {
    1
}

fn get_default_page_size() -> i32 {
    10
}

#[derive(Serialize, Debug)]
pub struct PaginatedResponse<T> {
    page: i32,
    page_size: i32,
    objects: Vec<T>,
}

impl<T> PaginatedResponse<T> {
    pub fn new(page: i32, page_size: i32, objects: Vec<T>) -> Self {
        Self {
            page,
            page_size,
            objects,
        }
    }
}
