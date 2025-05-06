#![allow(dead_code)]

pub struct Pack<T>(T);

impl<T> Pack<T> {
    pub fn unpack(self) -> T {
        self.0
    }
}

impl<T> From<T> for Pack<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> FromIterator<T> for Pack<Vec<T>> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}
