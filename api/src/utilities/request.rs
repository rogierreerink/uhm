pub mod collection {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Default)]
    pub struct GetResponse<T> {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub pagination: Option<Pagination>,
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
        pub skip: i64,
        pub take: i64,
        pub total: i64,
    }
}
