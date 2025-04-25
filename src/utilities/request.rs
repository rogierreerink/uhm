pub mod collection {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Default)]
    pub struct GetResponse<T> {
        pub data: Vec<T>,
    }

    #[derive(Deserialize)]
    pub struct PostRequest<T> {
        pub data: Vec<T>,
    }

    #[derive(Serialize, Default)]
    pub struct PostResponse<T> {
        pub data: Vec<T>,
    }

    #[derive(Serialize)]
    pub struct Pagination {
        pub skip: usize,
        pub take: usize,
        pub total: usize,
    }
}
