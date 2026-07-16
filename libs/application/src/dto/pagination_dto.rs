use serde::{Deserialize, Serialize};

/// Query string params cho mọi list endpoint dùng phân trang.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub q: Option<String>,
}

impl PaginationParams {
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn page_size(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 1000)
    }

    pub fn offset(&self) -> u32 {
        (self.page() - 1) * self.page_size()
    }

    pub fn search(&self) -> Option<&str> {
        self.q.as_deref().map(str::trim).filter(|s| !s.is_empty())
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}
