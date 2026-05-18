use lazy_regex::{Regex, RegexBuilder};
use serde::Deserialize;

/// Always returns `true`.
///
/// Useful for default values in rule configs that use serde.
/// See [serde documentation](https://serde.rs/field-attrs.html#default--path)
/// for more information
///
/// ## Example
/// ```ignore
/// use serde::Deserialize;
/// use oxc_linter::utils::default_true;
///
/// #[derive(Debug, Clone, Deserialize)]
/// pub struct RuleConfig {
///     // default to true
///     #[serde(default = "default_true")]
///     pub foo: bool,
///     // default to false
///     #[serde(default)]
///     pub bar: bool,
/// }
/// ```
#[inline]
pub const fn default_true() -> bool {
    true
}

pub fn deserialize_regex_option<'de, D>(deserializer: D) -> Result<Option<Regex>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    Option::<String>::deserialize(deserializer)?
        .map(|pattern| RegexBuilder::new(&pattern).build())
        .transpose()
        .map_err(D::Error::custom)
}
