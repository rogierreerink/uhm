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
    pub struct GetResponse<D> {
        pub pagination: Option<Pagination>,
        pub data: Vec<D>,
    }

    #[derive(Deserialize)]
    pub struct PostRequest<D> {
        pub data: Vec<D>,
    }

    #[derive(Serialize)]
    pub struct PostResponse<D> {
        pub data: Vec<D>,
    }
}
