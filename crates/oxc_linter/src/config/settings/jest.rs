use std::fmt;

use schemars::JsonSchema;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize};

use oxc_config::GlobSet;

/// Configure Jest plugin rules.
///
/// See [eslint-plugin-jest](https://github.com/jest-community/eslint-plugin-jest)'s
/// configuration for a full reference.
#[derive(Debug, Clone, Deserialize, Default, Serialize, JsonSchema, PartialEq, Eq)]
pub struct JestPluginSettings {
    /// Jest version — accepts a number (`29`) or a semver string (`"29.1.0"` or `"v29.1.0"`),
    /// storing only the major version.
    /// ::: warning
    /// Using this config will override the `no-deprecated-functions` config set.
    /// :::
    #[serde(default, deserialize_with = "jest_version_deserialize")]
    #[schemars(with = "Option<JestVersionSchema>")]
    pub version: Option<usize>,

    /// Extra glob patterns that mark a file as a Jest test file, on top of the built-in
    /// conventions (a `__tests__` directory, or a `.test.` / `.spec.` file name).
    ///
    /// Most Jest rules only run on test files. Use this for helpers that call `test()` or
    /// `expect()` under another name, such as BDD step definitions:
    ///
    /// ```json
    /// {
    ///   "settings": {
    ///     "jest": {
    ///       "additionalTestPatterns": ["**/*.steps.ts", "**/*.helper.ts"]
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// Patterns are matched against the file path relative to the directory holding the
    /// config file, the same way `overrides[].files` is matched. A pattern without a `/`
    /// is made recursive, so `"*.steps.ts"` and `"**/*.steps.ts"` are equivalent.
    ///
    /// Only adds files; it never stops a file from being recognized.
    #[serde(default, rename = "additionalTestPatterns")]
    pub additional_test_patterns: GlobSet,
}

#[derive(JsonSchema)]
#[serde(untagged)]
#[expect(dead_code)]
enum JestVersionSchema {
    Number(usize),
    String(String),
}

fn jest_version_deserialize<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    struct VersionVisitor;

    impl Visitor<'_> for VersionVisitor {
        type Value = Option<usize>;

        // Needed to impl, without this the deserialization will fail if null is provided.
        // The lack of this impl will cause to fail config test that use Oxlintrc default values.
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("Expected Jest version as a number or string")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            usize::try_from(v)
                .map(Some)
                .map_err(|_| E::custom(format!("Invalid Jest version integer: {v:?}")))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if v < 0 {
                return Err(E::custom(format!("Jest version cannot be negative: {v:?}")));
            }

            usize::try_from(v)
                .map(Some)
                .map_err(|_| E::custom(format!("Invalid Jest version integer: {v:?}")))
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let skip_v_prefix = usize::from(v.starts_with('v'));

            v.split('v')
                .nth(skip_v_prefix)
                .and_then(|semver| semver.split('.').next())
                .and_then(|s| s.parse::<usize>().ok())
                .map(Some)
                .ok_or_else(|| E::custom(format!("Invalid Jest version string: {v:?}")))
        }
    }

    deserializer.deserialize_any(VersionVisitor)
}
