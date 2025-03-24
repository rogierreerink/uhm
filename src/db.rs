use crate::types::error::Error;

pub mod ingredient_collections;
pub mod ingredients;
pub mod products;
pub mod shopping_list;

#[derive(Debug, Clone, PartialEq)]
pub enum DbError {
    NotFound,
    TooMany,
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DbError::NotFound => write!(f, "entry could not be found"),
            DbError::TooMany => write!(f, "query returned too many results"),
        }
    }
}

impl<T> From<DbError> for Error<DbError, T>
where
    T: std::error::Error,
{
    fn from(err: DbError) -> Self {
        Error::new(err)
    }
}

impl<T> From<T> for Error<DbError, T>
where
    T: std::error::Error,
{
    fn from(err: T) -> Error<DbError, T> {
        Error::from_error(err)
    }
}
