use std::{borrow::Cow, fmt};

use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_span::CompactStr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};

/// Regex to validate React version strings like "18.2.0", "17.0", or "16".
static REACT_VERSION_REGEX: Lazy<Regex> =
    lazy_regex!(r"^[1-9]\d*(\.(0|[1-9]\d*))?(\.(0|[1-9]\d*))?$");

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
    #[validate(regex = "REACT_VERSION_REGEX")]
    #[schemars(with = "Option<String>")]
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

    /// Checks if the React version supports `UNSAFE_` prefixed lifecycle methods.
    ///
    /// React 16.3 introduced the `UNSAFE_` prefixed lifecycle methods
    /// (`UNSAFE_componentWillMount`, `UNSAFE_componentWillReceiveProps`, `UNSAFE_componentWillUpdate`).
    ///
    /// Returns `true` if this version is >= 16.3.
    #[inline]
    pub fn supports_unsafe_lifecycle_prefix(&self) -> bool {
        self.major > 16 || (self.major == 16 && self.minor >= 3)
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
                    return Err(E::custom(
                        "version string cannot be empty for settings.react.version",
                    ));
                }

                let mut parts = value.split('.');
                let major = parts
                    .next()
                    .ok_or_else(|| E::custom("missing major version in settings.react.version"))?
                    .parse::<u32>()
                    .map_err(|_| E::custom("invalid major version in settings.react.version"))?;

                let minor = parts
                    .next()
                    .map(str::parse::<u32>)
                    .transpose()
                    .map_err(|_| E::custom("invalid minor version in settings.react.version"))?
                    .unwrap_or(0);

                let patch = parts
                    .next()
                    .map(str::parse::<u32>)
                    .transpose()
                    .map_err(|_| E::custom("invalid patch version in settings.react.version"))?
                    .unwrap_or(0);

                if parts.next().is_some() {
                    return Err(E::custom(
                        "version string in settings.react.version has too many components, expected major.minor.patch",
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

    #[test]
    fn test_supports_unsafe_lifecycle_prefix() {
        // Version 16.3.0 - should support UNSAFE_ prefix
        let v16_3: ReactVersion = serde_json::from_str(r#""16.3.0""#).unwrap();
        assert!(v16_3.supports_unsafe_lifecycle_prefix());

        // Version 16.4.0 - should support UNSAFE_ prefix
        let v16_4: ReactVersion = serde_json::from_str(r#""16.4.0""#).unwrap();
        assert!(v16_4.supports_unsafe_lifecycle_prefix());

        // Version 17.0.0 - should support UNSAFE_ prefix
        let v17: ReactVersion = serde_json::from_str(r#""17.0.0""#).unwrap();
        assert!(v17.supports_unsafe_lifecycle_prefix());

        // Version 16.2.0 - should NOT support UNSAFE_ prefix
        let v16_2: ReactVersion = serde_json::from_str(r#""16.2.0""#).unwrap();
        assert!(!v16_2.supports_unsafe_lifecycle_prefix());

        // Version 16.0.0 - should NOT support UNSAFE_ prefix
        let v16_0: ReactVersion = serde_json::from_str(r#""16.0.0""#).unwrap();
        assert!(!v16_0.supports_unsafe_lifecycle_prefix());

        // Version 15.0.0 - should NOT support UNSAFE_ prefix
        let v15: ReactVersion = serde_json::from_str(r#""15.0.0""#).unwrap();
        assert!(!v15.supports_unsafe_lifecycle_prefix());
    }

    #[test]
    fn test_version_regex() {
        let re = &*REACT_VERSION_REGEX;

        let valid_versions = vec!["18.2.0", "17.0", "16", "1.0.0", "10.20.30", "2.5"];
        for version in valid_versions {
            assert!(re.is_match(version), "Expected version '{version}' to match regex");
        }

        let invalid_versions = vec!["018.2.0", "17.-1", "18.2.0.1", "invalid", "", " "];
        for version in invalid_versions {
            assert!(!re.is_match(version), "Expected version '{version}' to not match regex");
        }
    }
}
