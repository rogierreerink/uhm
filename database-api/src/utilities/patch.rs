#![allow(dead_code)]

use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Patch<T> {
    Value(T),
    Null,
    #[serde(skip_serializing)]
    Undefined,
}

impl<T> Default for Patch<T> {
    fn default() -> Self {
        Self::Undefined
    }
}

impl<T> Patch<T> {
    pub fn as_ref(&self) -> Option<Option<&T>> {
        match self {
            Self::Value(value) => Some(Some(value)),
            Self::Null => Some(None),
            Self::Undefined => None,
        }
    }
    pub fn is_undefined(&self) -> bool {
        matches!(self, Patch::Undefined)
    }
}

impl<'de, T> Deserialize<'de> for Patch<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Deserialize::deserialize(deserializer) {
            Ok(Some(v)) => Ok(Patch::Value(v)),
            Ok(None) => Ok(Patch::Null),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Foo {
        bar: Patch<bool>,
        baz: Patch<String>,
        cas: Patch<Coo>,
    }

    #[derive(Deserialize, Default, Debug, PartialEq)]
    struct Coo {
        car: bool,
        caz: u32,
    }

    #[derive(Serialize)]
    struct Boo {
        bar: Patch<bool>,
    }

    #[test]
    fn deserialize_value() {
        let foo: Foo = serde_json::from_value(json!({
            "bar": true,
            "baz": "hello, world",
            "cas": {
                "car": false,
                "caz": 123
            },
        }))
        .unwrap();
        assert_eq!(foo.bar, Patch::Value(true));
        assert_eq!(foo.baz, Patch::Value("hello, world".to_string()));
        assert_eq!(
            foo.cas,
            Patch::Value(Coo {
                car: false,
                caz: 123
            })
        );
    }

    #[test]
    fn deserialize_null() {
        let foo: Foo = serde_json::from_value(json!({
            "bar": null,
            "baz": null,
            "cas": null,
        }))
        .unwrap();
        assert_eq!(foo.bar, Patch::Null);
        assert_eq!(foo.baz, Patch::Null);
        assert_eq!(foo.cas, Patch::Null);
    }

    #[test]
    fn deserialize_undefined() {
        let foo: Foo = serde_json::from_value(json!({})).unwrap();
        assert_eq!(foo.bar, Patch::Undefined);
        assert_eq!(foo.baz, Patch::Undefined);
        assert_eq!(foo.cas, Patch::Undefined);
    }

    #[test]
    fn as_ref() {
        let patch = Patch::Value(true);
        assert_eq!(patch.as_ref(), Some(Some(&true)));
    }

    #[test]
    fn serialize_value() {
        let boo = Boo {
            bar: Patch::Value(true),
        };
        assert_eq!("{\"bar\":true}", serde_json::to_string(&boo).unwrap());
    }

    #[test]
    fn serialize_null() {
        let boo = Boo {
            bar: Patch::Null::<bool>,
        };
        assert_eq!("{\"bar\":null}", serde_json::to_string(&boo).unwrap());
    }
}
