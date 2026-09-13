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
    /// conventions: a `__tests__` path segment, or a file name whose second-to-last
    /// dot-separated segment is `test` or `spec` (`foo.test.ts`, `foo.spec.tsx`).
    ///
    /// Most Jest rules only run on files recognized as test files, so a helper module that
    /// calls `test()` or `expect()` under a project-specific name — BDD step definitions,
    /// shared assertion helpers — is skipped by most of them. List such files here to lint
    /// them:
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
    /// Purely additive: it never stops a file from being recognized, and it takes effect only
    /// when the `jest` or `vitest` plugin is enabled. Use
    /// `settings.vitest.additionalTestPatterns` to mark a file as Vitest instead.
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
