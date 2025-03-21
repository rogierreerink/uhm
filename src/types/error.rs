use std::fmt::{Debug, Display, Formatter};

pub enum Error<K, T>
where
    K: Debug + Display + PartialEq<K>,
    T: std::error::Error,
{
    New(K),
    Inner(T),
}

impl<K, T> Error<K, T>
where
    K: Debug + Display + PartialEq<K>,
    T: std::error::Error,
{
    pub fn new(kind: K) -> Error<K, T> {
        Error::New(kind)
    }
    pub fn from_error(inner: T) -> Error<K, T> {
        Error::Inner(inner)
    }
}

impl<K, T> Debug for Error<K, T>
where
    K: Debug + Display + PartialEq<K>,
    T: std::error::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::New(e) => write!(f, "{:?}", e),
            Error::Inner(e) => write!(f, "{:?}", e),
        }
    }
}

impl<K, T> Display for Error<K, T>
where
    K: Debug + Display + PartialEq<K>,
    T: std::error::Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::New(e) => write!(f, "{}", e),
            Error::Inner(e) => write!(f, "{}", e),
        }
    }
}

impl<K, T> PartialEq<K> for Error<K, T>
where
    K: Debug + Display + PartialEq<K>,
    T: std::error::Error,
{
    fn eq(&self, other: &K) -> bool {
        match self {
            Error::New(err) if err == other => true,
            _ => false,
        }
    }
}
