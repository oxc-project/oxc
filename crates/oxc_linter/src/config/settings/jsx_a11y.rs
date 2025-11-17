use oxc_span::CompactStr;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configure JSX A11y plugin rules.
///
/// See
/// [eslint-plugin-jsx-a11y](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y#configurations)'s
/// configuration for a full reference.
#[derive(Debug, Clone, Deserialize, Default, Serialize, JsonSchema, PartialEq, Eq)]
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

    /// Map of attribute names to their DOM equivalents.
    /// This is useful for non-React frameworks that use different attribute names.
    ///
    /// Example:
    ///
    /// ```json
    /// {
    ///   "settings": {
    ///     "jsx-a11y": {
    ///       "attributes": {
    ///         "for": ["htmlFor", "for"]
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    #[serde(default)]
    pub attributes: FxHashMap<CompactStr, Vec<CompactStr>>,
}

impl JSXA11yPluginSettings {
    /// Deep merge self into other (self takes priority)
    pub(crate) fn merge(mut self, other: Self) -> Self {
        // If self has polymorphic_prop_name, use it; otherwise use other's
        if self.polymorphic_prop_name.is_none() {
            self.polymorphic_prop_name = other.polymorphic_prop_name;
        }

        // Deep merge components: other's entries + self's entries (self overrides)
        for (key, value) in other.components {
            self.components.entry(key).or_insert(value);
        }

        // Deep merge attributes: other's entries + self's entries (self overrides)
        for (key, value) in other.attributes {
            self.attributes.entry(key).or_insert(value);
        }

        self
    }

    /// Deep merge self into base (self takes priority), mutating base in place
    pub(crate) fn merge_into(&self, base: &mut Self) {
        // If self has polymorphic_prop_name, override base's
        if let Some(ref prop_name) = self.polymorphic_prop_name {
            base.polymorphic_prop_name = Some(prop_name.clone());
        }

        // Deep merge components: self's entries override base's
        for (key, value) in &self.components {
            base.components.insert(key.clone(), value.clone());
        }

        // Deep merge attributes: self's entries override base's
        for (key, value) in &self.attributes {
            base.attributes.insert(key.clone(), value.clone());
        }
    }
}
