use oxc_span::CompactStr;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configure JSX A11y plugin rules.
///
/// See
/// [eslint-plugin-jsx-a11y](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y#configurations)'s
/// configuration for a full reference.
#[derive(Debug, Clone, Deserialize, Default, Serialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
pub struct JSXA11yPluginSettings {
    /// An optional setting that define the prop your code uses to create polymorphic components.
    /// This setting will be used to determine the element type in rules that
    /// require semantic context.
    ///
    /// For example, if you set the `polymorphicPropName` to `as`, then this element:
    ///
    /// ```jsx
    /// <Box as="h3">Hello</Box>
    /// ```
    ///
    /// Will be treated as an `h3`. If not set, this component will be treated
    /// as a `Box`.
    #[serde(rename = "polymorphicPropName")]
    pub polymorphic_prop_name: Option<CompactStr>,

    /// To have your custom components be checked as DOM elements, you can
    /// provide a mapping of your component names to the DOM element name.
    ///
    /// Example:
    ///
    /// ```json
    /// {
    ///   "settings": {
    ///     "jsx-a11y": {
    ///       "components": {
    ///         "Link": "a",
    ///         "IconButton": "button"
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    #[serde(default)]
    pub components: FxHashMap<CompactStr, CompactStr>,
}
