use serde::{Deserialize, Serialize};

pub trait Modifier {
    type Key<T>;
    type Meta<T>;
    type Data<T>;
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
}
