use std::fmt;

use rustc_hash::{FxHashMap, FxHashSet};

use nonmax::NonMaxU32;
use oxc_index::{Idx, IndexVec};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExternalPluginId(NonMaxU32);

impl Idx for ExternalPluginId {
    #[expect(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is valid for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExternalRuleId(NonMaxU32);

impl Idx for ExternalRuleId {
    #[expect(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is valid for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

impl ExternalRuleId {
    #[inline]
    pub fn as_u32(self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug, Default)]
pub struct ExternalPluginStore {
    registered_plugin_specifiers: FxHashSet<String>,

    plugins: IndexVec<ExternalPluginId, ExternalPlugin>,
    plugin_names: FxHashMap<String, ExternalPluginId>,
    rules: IndexVec<ExternalRuleId, ExternalRule>,
}

impl ExternalPluginStore {
    pub fn is_plugin_registered(&self, plugin_specifier: &str) -> bool {
        self.registered_plugin_specifiers.contains(plugin_specifier)
    }

    /// # Panics
    /// Panics if you use it wrong
    pub fn register_plugin(
        &mut self,
        plugin_specifier: String,
        plugin_name: String,
        offset: usize,
        rules: Vec<String>,
    ) {
        let registered_plugin_specifier_newly_inserted =
            self.registered_plugin_specifiers.insert(plugin_specifier);
        assert!(registered_plugin_specifier_newly_inserted);

        let plugin_id: ExternalPluginId = self
            .plugins
            .push(ExternalPlugin { name: plugin_name.clone(), rules: FxHashMap::default() });
        self.plugin_names.insert(plugin_name, plugin_id);

        assert!(
            offset == self.rules.len(),
            "register_plugin: expected offset {}, but rule table is currently {} long",
            offset,
            self.rules.len()
        );

        for rule_name in rules {
            let rule_id = self.rules.push(ExternalRule { name: rule_name.clone(), plugin_id });
            self.plugins[plugin_id].rules.insert(rule_name, rule_id);
        }
    }

    /// # Errors
    /// Returns an error if the plugin, or rule could not be found
    pub fn lookup_rule_id(
        &self,
        plugin_name: &str,
        rule_name: &str,
    ) -> Result<ExternalRuleId, ExternalRuleLookupError> {
        let plugin_id: &ExternalPluginId = self.plugin_names.get(plugin_name).ok_or_else(|| {
            ExternalRuleLookupError::PluginNotFound { plugin: plugin_name.to_string() }
        })?;

        self.plugins[*plugin_id].rules.get(rule_name).copied().ok_or_else(|| {
            ExternalRuleLookupError::RuleNotFound {
                plugin: plugin_name.to_string(),
                rule: rule_name.to_string(),
            }
        })
    }

    pub fn resolve_plugin_rule_names(&self, external_rule_id: u32) -> Option<(&str, &str)> {
        let external_rule =
            self.rules.get(NonMaxU32::new(external_rule_id).map(ExternalRuleId)?)?;
        let plugin = &self.plugins[external_rule.plugin_id];

        Some((&plugin.name, &external_rule.name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalRuleLookupError {
    PluginNotFound { plugin: String },
    RuleNotFound { plugin: String, rule: String },
}

impl fmt::Display for ExternalRuleLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExternalRuleLookupError::PluginNotFound { plugin } => {
                write!(f, "Plugin '{plugin}' not found",)
            }
            ExternalRuleLookupError::RuleNotFound { plugin, rule } => {
                write!(f, "Rule '{rule}' not found in plugin '{plugin}'")
            }
        }
    }
}

impl std::error::Error for ExternalRuleLookupError {}

#[derive(Debug, Default)]
struct ExternalPlugin {
    name: String,
    rules: FxHashMap<String, ExternalRuleId>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ExternalRule {
    name: String,
    plugin_id: ExternalPluginId,
}
