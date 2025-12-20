use std::{borrow::Cow, fmt};

use oxc_span::CompactStr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};

/// Configure React plugin rules.
///
/// Derived from [eslint-plugin-react](https://github.com/jsx-eslint/eslint-plugin-react#configuration-legacy-eslintrc-)
#[derive(Debug, Clone, Deserialize, Default, Serialize, JsonSchema, PartialEq)]
pub struct ReactPluginSettings {
    /// Components used as alternatives to `<form>` for forms, such as `<Formik>`.
    ///
    /// Example:
    ///
    /// ```jsonc
    /// {
    ///   "settings": {
    ///     "react": {
    ///       "formComponents": [
    ///         "CustomForm",
    ///         // OtherForm is considered a form component and has an endpoint attribute
    ///         { "name": "OtherForm", "formAttribute": "endpoint" },
    ///         // allows specifying multiple properties if necessary
    ///         { "name": "Form", "formAttribute": ["registerEndpoint", "loginEndpoint"] }
    ///       ]
    ///     }
    ///   }
    /// }
    /// ```
    #[serde(default)]
    #[serde(rename = "formComponents")]
    form_components: Vec<CustomComponent>,

    /// Components used as alternatives to `<a>` for linking, such as `<Link>`.
    ///
    /// Example:
    ///
    /// ```jsonc
    /// {
    ///   "settings": {
    ///     "react": {
    ///       "linkComponents": [
    ///         "HyperLink",
    ///         // Use `linkAttribute` for components that use a different prop name
    ///         // than `href`.
    ///         { "name": "MyLink", "linkAttribute": "to" },
    ///         // allows specifying multiple properties if necessary
    ///         { "name": "Link", "linkAttribute": ["to", "href"] }
    ///       ]
    ///     }
    ///   }
    /// }
    /// ```
    #[serde(default)]
    #[serde(rename = "linkComponents")]
    link_components: Vec<CustomComponent>,

    /// React version to use for version-specific rules.
    ///
    /// Accepts semver versions (e.g., "18.2.0", "17.0").
    ///
    /// Example:
    ///
    /// ```jsonc
    /// {
    ///   "settings": {
    ///     "react": {
    ///       "version": "18.2.0"
    ///     }
    ///   }
    /// }
    /// ```
    #[serde(default)]
    pub version: Option<ReactVersion>,
    // TODO: More properties should be added
}
pub type ComponentAttrs<'c> = Cow<'c, Vec<CompactStr>>;
impl ReactPluginSettings {
    pub fn get_form_component_attrs(&self, name: &str) -> Option<ComponentAttrs<'_>> {
        get_component_attrs_by_name(&self.form_components, name)
    }

    pub fn get_link_component_attrs(&self, name: &str) -> Option<ComponentAttrs<'_>> {
        get_component_attrs_by_name(&self.link_components, name)
    }
}

// Deserialize helper types

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(untagged)]
enum CustomComponent {
    NameOnly(CompactStr),
    ObjectWithOneAttr {
        name: CompactStr,
        #[serde(alias = "formAttribute", alias = "linkAttribute")]
        attribute: CompactStr,
    },
    ObjectWithManyAttrs {
        name: CompactStr,
        #[serde(alias = "formAttribute", alias = "linkAttribute")]
        attributes: Vec<CompactStr>,
    },
}

fn get_component_attrs_by_name<'c>(
    components: &'c Vec<CustomComponent>,
    name: &str,
) -> Option<ComponentAttrs<'c>> {
    for item in components {
        match item {
            CustomComponent::NameOnly(comp_name) if comp_name == name => {
                return Some(Cow::Owned(vec![]));
            }
            CustomComponent::ObjectWithOneAttr { name: comp_name, attribute }
                if comp_name == name =>
            {
                return Some(Cow::Owned(vec![attribute.clone()]));
            }
            CustomComponent::ObjectWithManyAttrs { name: comp_name, attributes }
                if comp_name == name =>
            {
                return Some(Cow::Borrowed(attributes));
            }
            _ => {}
        }
    }

    None
}

/// React version parsed into (major, minor, patch) components.
///
/// Supports versions like "18.2.0", "17.0", or "16".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReactVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl ReactVersion {
    #[inline]
    pub fn major(&self) -> u32 {
        self.major
    }

    #[inline]
    pub fn minor(&self) -> u32 {
        self.minor
    }

    #[inline]
    pub fn patch(&self) -> u32 {
        self.patch
    }
}

impl fmt::Display for ReactVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Serialize for ReactVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ReactVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ReactVersionVisitor;

        impl de::Visitor<'_> for ReactVersionVisitor {
            type Value = ReactVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(r#"a version string like "18.2.0" or "17.0""#)
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                if value.is_empty() {
                    return Err(E::custom("version string cannot be empty"));
                }

                let mut parts = value.split('.');
                let major = parts
                    .next()
                    .ok_or_else(|| E::custom("missing major version"))?
                    .parse::<u32>()
                    .map_err(|_| E::custom("invalid major version"))?;

                let minor = parts
                    .next()
                    .map(str::parse::<u32>)
                    .transpose()
                    .map_err(|_| E::custom("invalid minor version"))?
                    .unwrap_or(0);

                let patch = parts
                    .next()
                    .map(str::parse::<u32>)
                    .transpose()
                    .map_err(|_| E::custom("invalid patch version"))?
                    .unwrap_or(0);

                if parts.next().is_some() {
                    return Err(E::custom(
                        "version string has too many components, expected major.minor.patch",
                    ));
                }

                Ok(ReactVersion { major, minor, patch })
            }
        }

        deserializer.deserialize_str(ReactVersionVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_react_version_deserialize() {
        let v: ReactVersion = serde_json::from_str(r#""18.2.0""#).unwrap();
        assert_eq!(v.major(), 18);
        assert_eq!(v.minor(), 2);
        assert_eq!(v.patch(), 0);

        let v: ReactVersion = serde_json::from_str(r#""17.0""#).unwrap();
        assert_eq!(v.major(), 17);
        assert_eq!(v.minor(), 0);
        assert_eq!(v.patch(), 0);

        let v: ReactVersion = serde_json::from_str(r#""16""#).unwrap();
        assert_eq!(v.major(), 16);
        assert_eq!(v.minor(), 0);
        assert_eq!(v.patch(), 0);

        let v: ReactVersion = serde_json::from_str(r#""018.002.001""#).unwrap();
        assert_eq!(v.major(), 18);
        assert_eq!(v.minor(), 2);
        assert_eq!(v.patch(), 1);
    }

    #[test]
    fn test_react_version_serialize() {
        let v = ReactVersion { major: 18, minor: 2, patch: 1 };
        assert_eq!(serde_json::to_string(&v).unwrap(), r#""18.2.1""#);
    }

    #[test]
    fn test_react_version_display() {
        let v = ReactVersion { major: 18, minor: 2, patch: 1 };
        assert_eq!(v.to_string(), "18.2.1");
    }

    #[test]
    fn test_react_version_invalid() {
        assert!(serde_json::from_str::<ReactVersion>(r#""invalid""#).is_err());
        assert!(serde_json::from_str::<ReactVersion>(r#""18.x.0""#).is_err());
        assert!(serde_json::from_str::<ReactVersion>(r#""""#).is_err());
        assert!(serde_json::from_str::<ReactVersion>(r#""18.2.0.1""#).is_err());
        assert!(serde_json::from_str::<ReactVersion>(r#""18.2.0.1.2""#).is_err());
        assert!(serde_json::from_str::<ReactVersion>(r#"" 18.2.0""#).is_err());
        assert!(serde_json::from_str::<ReactVersion>(r#""18.2.0 ""#).is_err());
        assert!(serde_json::from_str::<ReactVersion>(r#""18. 2.0""#).is_err());
    }
}
