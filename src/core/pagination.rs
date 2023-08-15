use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PaginatedResponse<T> {
    page: usize,
    page_size: usize,
    objects: Vec<T>,
}

impl<T> PaginatedResponse<T> {
    pub fn new(page: usize, page_size: usize, objects: Vec<T>) -> Self {
        Self {
            page,
            page_size,
            objects,
        }
    }
}
