use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod collection {
    use super::*;

    #[derive(Serialize)]
    pub struct Pagination {
        pub skip: usize,
        pub take: usize,
        pub total: usize,
    }

    #[derive(Serialize)]
    pub struct GetResponse<I, D> {
        pub pagination: Option<Pagination>,
        pub data: Vec<resource::GetResponse<I, D>>,
    }

    #[derive(Deserialize)]
    pub struct PostRequest<D> {
        pub data: Vec<D>,
    }

    #[derive(Serialize)]
    pub struct PostResponse<I> {
        pub data: Vec<resource::PostResponse<I>>,
    }
}

pub mod resource {
    use super::*;

    #[derive(Serialize)]
    pub struct GetResponse<I, D> {
        pub id: I,
        pub created: DateTime<Utc>,
        pub updated: Option<DateTime<Utc>>,
        pub data: D,
    }

    #[derive(Serialize)]
    pub struct PostResponse<I> {
        pub id: I,
    }
}
