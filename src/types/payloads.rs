use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CollectionRequest<D> {
    pub data: Vec<D>,
}

#[derive(Serialize)]
pub struct CollectionResponse<I, D> {
    pub pagination: Option<Pagination>,
    pub data: Vec<ResourceResponse<I, D>>,
}

#[derive(Serialize)]
pub struct ResourceResponse<I, D> {
    pub id: I,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
    pub data: D,
}

#[derive(Serialize)]
pub struct Pagination {
    pub skip: usize,
    pub take: usize,
    pub total: usize,
}
