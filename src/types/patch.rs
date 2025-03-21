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

    #[derive(Serialize)]
    struct Boo {
        bar: Patch<bool>,
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

// impl<T> Into<Option<T>> for Patch<T> {
//     fn into(self) -> Option<T> {
//         match self {
//             Patch::Value(v) => Some(v),
//             Patch::Null => None,
//             Patch::Undefined => None,
//         }
//     }
// }
//
// impl<T> From<Option<T>> for Patch<T> {
//     fn from(option: Option<T>) -> Self {
//         option.map(Patch::Value).unwrap_or(Patch::Null)
//     }
// }
//
// impl<T> Into<Option<Option<T>>> for Patch<T> {
//     fn into(self) -> Option<Option<T>> {
//         match self {
//             Patch::Value(v) => Some(Some(v)),
//             Patch::Null => Some(None),
//             Patch::Undefined => None,
//         }
//     }
// }
//
// impl<T> From<Option<Option<T>>> for Patch<T> {
//     fn from(double_option: Option<Option<T>>) -> Self {
//         double_option
//             .map(|v| v.map(Patch::Value).unwrap_or(Patch::Null))
//             .unwrap_or(Patch::Undefined)
//     }
// }

// #[test]
// fn into_single_option() {
//     let opt = Some(true);
//     assert_eq!(opt, Patch::Value(true).into());
//     let opt = None::<bool>;
//     assert_eq!(opt, Patch::Null.into());
// }
//
// #[test]
// fn into_double_option() {
//     let opt: Option<Option<_>> = Some(Some(true));
//     assert_eq!(opt, Patch::Value(true).into());
//     let opt: Option<Option<_>> = Some(None);
//     assert_eq!(opt, Patch::<bool>::Null.into());
//     let opt: Option<Option<_>> = None;
//     assert_eq!(opt, Patch::<bool>::Undefined.into());
// }
