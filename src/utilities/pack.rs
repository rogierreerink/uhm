pub struct Pack<T>(T);

impl<T> Pack<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn unpack(self) -> T {
        self.0
    }
}
