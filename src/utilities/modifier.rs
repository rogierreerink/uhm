use serde::{Deserialize, Serialize};

pub trait Modifier {
    type Key<T>;
    type Meta<T>;
    type Data<T>;

    fn skip_meta<T>(_: &Self::Meta<T>) -> bool {
        false
    }
    fn skip_data<T>(_: &Self::Data<T>) -> bool {
        false
    }
}

#[derive(Debug, Serialize)]
pub struct Query;

impl Modifier for Query {
    type Key<T> = T;
    type Meta<T> = T;
    type Data<T> = T;
}

#[derive(Default, Debug, Deserialize)]
pub struct Create;

impl Modifier for Create {
    type Key<T> = ();
    type Meta<T> = ();
    type Data<T> = T;
}

#[derive(Default, Debug, Deserialize)]
pub struct Update;

impl Modifier for Update {
    type Key<T> = ();
    type Meta<T> = ();
    type Data<T> = Option<T>;
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Reference;

impl Modifier for Reference {
    type Key<T> = T;
    type Meta<T> = Option<T>;
    type Data<T> = Option<T>;

    fn skip_meta<T>(value: &Self::Meta<T>) -> bool {
        value.is_none()
    }
    fn skip_data<T>(value: &Self::Data<T>) -> bool {
        value.is_none()
    }
}
