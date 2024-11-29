use std::{borrow, fmt, hash};

use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{de::Visitor, Deserialize, Serialize};

/// Add or remove global variables.
///
/// For each global variable, set the corresponding value equal to `"writable"`
/// to allow the variable to be overwritten or `"readonly"` to disallow overwriting.
///
/// Globals can be disabled by setting their value to `"off"`. For example, in
/// an environment where most Es2015 globals are available but `Promise` is unavailable,
/// you might use this config:
///
/// ```json
///
/// {
///     "$schema": "./node_modules/oxlint/configuration_schema.json",
///     "env": {
///         "es6": true
///     },
///     "globals": {
///         "Promise": "off"
///     }
/// }
///
/// ```
///
/// You may also use `"readable"` or `false` to represent `"readonly"`, and
/// `"writeable"` or `true` to represent `"writable"`.
// <https://eslint.org/docs/v8.x/use/configure/language-options#using-configuration-files-1>
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema, Clone)]
pub struct OxlintGlobals(FxHashMap<String, GlobalValue>);
impl OxlintGlobals {
    pub fn is_enabled<Q>(&self, name: &Q) -> bool
    where
        String: borrow::Borrow<Q>,
        Q: ?Sized + Eq + hash::Hash,
    {
        self.0.get(name).is_some_and(|value| *value != GlobalValue::Off)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum GlobalValue {
    Readonly,
    Writeable,
    Off,
}

impl GlobalValue {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Readonly => "readonly",
            Self::Writeable => "writeable",
            Self::Off => "off",
        }
    }
}

impl<'de> Deserialize<'de> for GlobalValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(GlobalValueVisitor)
    }
}

impl From<bool> for GlobalValue {
    #[inline]
    fn from(value: bool) -> Self {
        if value {
            GlobalValue::Writeable
        } else {
            GlobalValue::Readonly
        }
    }
}

impl TryFrom<&str> for GlobalValue {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "readonly" | "readable" => Ok(GlobalValue::Readonly),
            "writable" | "writeable" => Ok(GlobalValue::Writeable),
            "off" => Ok(GlobalValue::Off),
            _ => Err("Invalid global value"),
        }
    }
}

impl fmt::Display for GlobalValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

struct GlobalValueVisitor;
impl Visitor<'_> for GlobalValueVisitor {
    type Value = GlobalValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "'readonly', 'writable', 'off', or a boolean")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.into())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.try_into().map_err(E::custom)
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    macro_rules! globals {
        ($($json:tt)+) => {
            OxlintGlobals::deserialize(&json!($($json)+)).unwrap()
        };
    }

    #[test]
    fn test_deserialize_normal() {
        let globals = globals!({
            "foo": "readonly",
            "bar": "writable",
            "baz": "off",
        });
        assert!(globals.is_enabled("foo"));
        assert!(globals.is_enabled("bar"));
        assert!(!globals.is_enabled("baz"));
    }

    #[test]
    fn test_deserialize_legacy_spelling() {
        let globals = globals!({
            "foo": "readable",
            "bar": "writeable",
        });
        assert!(globals.is_enabled("foo"));
        assert!(globals.is_enabled("bar"));
    }

    #[test]
    fn test_deserialize_bool() {
        let globals = globals!({
            "foo": true,
            "bar": false,
        });
        assert!(globals.is_enabled("foo"));
        assert!(globals.is_enabled("bar"));
    }
}
