use std::fmt;

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_index::{IndexVec, define_index_type};

define_index_type! {
    pub struct ExternalPluginId = u32;
}

define_index_type! {
    pub struct ExternalRuleId = u32;
}

#[derive(Debug, Default)]
pub struct ExternalPluginStore {
    registered_plugin_paths: FxHashSet<String>,

    plugins: IndexVec<ExternalPluginId, ExternalPlugin>,
    plugin_names: FxHashMap<String, ExternalPluginId>,
    rules: IndexVec<ExternalRuleId, ExternalRule>,
}

impl ExternalPluginStore {
    pub fn is_plugin_registered(&self, plugin_path: &str) -> bool {
        self.registered_plugin_paths.contains(plugin_path)
    }

    /// Register plugin.
    ///
    /// # Panics
    /// Panics if:
    /// - Plugin at `plugin_path` is already registered.
    /// - `offset` does not equal the number of registered rules.
    pub fn register_plugin(
        &mut self,
        plugin_path: String,
        plugin_name: String,
        offset: usize,
        rules: Vec<String>,
    ) {
        let newly_inserted = self.registered_plugin_paths.insert(plugin_path);
        assert!(newly_inserted, "register_plugin: plugin already registered");

        let plugin_id = self
            .plugins
            .push(ExternalPlugin { name: plugin_name.clone(), rules: FxHashMap::default() });
        self.plugin_names.insert(plugin_name, plugin_id);

        assert!(
            offset == self.rules.len(),
            "register_plugin: received offset {}, but rule table is currently {} long",
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
        let plugin_id = *self.plugin_names.get(plugin_name).ok_or_else(|| {
            ExternalRuleLookupError::PluginNotFound { plugin: plugin_name.to_string() }
        })?;

        self.plugins[plugin_id].rules.get(rule_name).copied().ok_or_else(|| {
            ExternalRuleLookupError::RuleNotFound {
                plugin: plugin_name.to_string(),
                rule: rule_name.to_string(),
            }
        })
    }

    pub fn resolve_plugin_rule_names(&self, external_rule_id: u32) -> Option<(&str, &str)> {
        let external_rule = self.rules.get(ExternalRuleId::from_raw(external_rule_id))?;
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

#[derive(Debug)]
struct ExternalPlugin {
    name: String,
    rules: FxHashMap<String, ExternalRuleId>,
}

#[derive(Debug)]
struct ExternalRule {
    name: String,
    plugin_id: ExternalPluginId,
}
