#[derive(Debug, PartialEq)]
pub struct PaginationParams {
    pub(crate) page: i32,
    pub(crate) page_size: i32,
}

impl PaginationParams {
    pub fn new(page: i32, page_size: i32) -> Self {
        let page = match page {
            value if value < 0 => 1,
            _ => page,
        };

        let page_size = match page_size {
            value if value < 0 => 1,
            _ => page_size,
        };

        Self { page, page_size }
    }
}
