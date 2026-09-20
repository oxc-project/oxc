use std::{
    fmt,
    path::{Path, PathBuf},
};

use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

use oxc_index::{IndexVec, define_index_type, index_vec};
use serde::{Serialize, Serializer};

use crate::ExternalLinter;

define_index_type! {
    pub struct ExternalPluginId = u32;
    MAX_INDEX = u32::MAX as usize;
}

define_index_type! {
    pub struct ExternalRuleId = u32;
    MAX_INDEX = u32::MAX as usize;
}

impl ExternalRuleId {
    /// Dummy value used in first element of `ExternalPluginStore::options`, which is a dummy
    pub const DUMMY: Self = Self::from_usize(0);
}

define_index_type! {
    pub struct ExternalOptionsId = u32;
    MAX_INDEX = u32::MAX as usize;
}

impl ExternalOptionsId {
    /// The value `0`.
    /// Used as the ID when a rule does not have options.
    pub const NONE: Self = Self::from_usize(0);
}

/// Identity of the plugin a config asks to load: package name, version and entry point.
///
/// The same package can be installed at several paths: npm and Yarn with the `node_modules` linker
/// nest a copy under each dependent which cannot share a hoisted one, and pnpm gives each peer
/// dependency variant of a version its own store entry. Each copy has its own realpath, so
/// deduplication by path does not recognise them as one plugin. The identity is read from the
/// plugin's own `package.json`, which every copy carries unchanged. Only plugins installed under
/// `node_modules` get an identity.
///
/// The same name and version is treated as the same code, so a patched copy at the same version
/// (`pnpm patch`, a `file:` install) is merged with the unpatched one.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ExternalPluginIdentity {
    pub package_name: String,
    pub version: String,
    /// Relative to the directory containing the package's `package.json`.
    /// A package can contain several plugins, which all share its `package.json`.
    pub entry_path: PathBuf,
}

/// Name a plugin was registered under.
#[derive(Debug)]
struct RegisteredName {
    name: String,
    /// The name was given by an alias in the config.
    is_alias: bool,
}

/// Store of the external (JS) plugins loaded for one workspace, and of their rules and options.
///
/// # Semantics
///
/// A plugin whose resolved path is not registered is reused when another installation of it is
/// registered: same package name, version and entry point.
///
/// 1. A request without an alias reuses a registration which had no alias. A request with an alias
///    reuses a registration whose name is that alias. Any other request is loaded, because the
///    alias decides the name a plugin is registered under. In particular a request without an
///    alias does not reuse an aliased registration, even one aliased to the plugin's own name, so
///    such a mix can be reported as a clash by the JS side depending on the load order. A request
///    for a registered path is reused whatever its alias, as
///    [`ExternalPluginStore::is_plugin_registered`] is keyed by path only.
/// 2. A plugin without a `name` or a `version` in its `package.json`, or which is not installed
///    under `node_modules`, has no identity and is never reused. It is loaded, and a name clash
///    is reported by the JS side.
/// 3. Two versions of a package are different plugins. Both are loaded, and a name clash is
///    reported by the JS side.
#[derive(Debug)]
pub struct ExternalPluginStore {
    registered_plugin_paths: FxHashSet<PathBuf>,
    /// Names the plugins of each identity are registered under.
    registered_plugin_identities: FxHashMap<ExternalPluginIdentity, Vec<RegisteredName>>,

    plugins: IndexVec<ExternalPluginId, ExternalPlugin>,
    plugin_names: FxHashMap<String, ExternalPluginId>,
    rules: IndexVec<ExternalRuleId, ExternalRule>,
    /// Options for a rule, indexed by `ExternalOptionsId`.
    /// The rule ID is also stored, so that can merge options with the rule's default options on JS side.
    options: IndexVec<ExternalOptionsId, (ExternalRuleId, SmallVec<[serde_json::Value; 1]>)>,

    is_enabled: bool,
}

impl Default for ExternalPluginStore {
    fn default() -> Self {
        Self::new(true)
    }
}

impl ExternalPluginStore {
    pub fn new(is_enabled: bool) -> Self {
        let options = index_vec![(ExternalRuleId::DUMMY, SmallVec::new())];

        Self {
            registered_plugin_paths: FxHashSet::default(),
            registered_plugin_identities: FxHashMap::default(),
            plugins: IndexVec::default(),
            plugin_names: FxHashMap::default(),
            rules: IndexVec::default(),
            options,
            is_enabled,
        }
    }

    /// Returns `true` if external plugins are enabled.
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// Returns `true` if no external plugins have been loaded.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    pub fn is_plugin_registered(&self, plugin_path: &Path) -> bool {
        self.registered_plugin_paths.contains(plugin_path)
    }

    /// Returns `true` if the plugin at `plugin_path` is another installation of a registered plugin
    /// and is reused, so the caller must not load it on JS side.
    /// The path is recorded, so [`ExternalPluginStore::is_plugin_registered`] matches it later.
    pub fn try_reuse_plugin(
        &mut self,
        plugin_path: &Path,
        identity: Option<&ExternalPluginIdentity>,
        alias: Option<&str>,
    ) -> bool {
        let Some(identity) = identity else { return false };
        let Some(registered_names) = self.registered_plugin_identities.get(identity) else {
            return false;
        };

        let is_reused = registered_names.iter().any(|registered| match alias {
            Some(alias) => registered.name == alias,
            None => !registered.is_alias,
        });
        if is_reused {
            self.registered_plugin_paths.insert(plugin_path.to_path_buf());
        }
        is_reused
    }

    /// Register plugin.
    ///
    /// # Panics
    /// Panics if:
    /// - Plugin at `plugin_path` is already registered.
    /// - `offset` does not equal the number of registered rules.
    pub fn register_plugin(
        &mut self,
        plugin_path: PathBuf,
        plugin_name: String,
        is_alias: bool,
        identity: Option<ExternalPluginIdentity>,
        offset: usize,
        rule_names: Vec<String>,
    ) {
        let newly_inserted = self.registered_plugin_paths.insert(plugin_path);
        assert!(newly_inserted, "register_plugin: plugin already registered");

        if let Some(identity) = identity {
            self.registered_plugin_identities
                .entry(identity)
                .or_default()
                .push(RegisteredName { name: plugin_name.clone(), is_alias });
        }

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

        for rule_name in rule_names {
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

    pub fn resolve_plugin_rule_names(
        &self,
        external_rule_id: ExternalRuleId,
    ) -> (/* plugin name */ &str, /* rule name */ &str) {
        let external_rule = &self.rules[external_rule_id];
        let plugin = &self.plugins[external_rule.plugin_id];
        (&plugin.name, &external_rule.name)
    }

    /// Add options to the store and return its [`ExternalOptionsId`].
    /// If `options` is empty, returns [`ExternalOptionsId::NONE`] without adding to the store.
    pub fn add_options(
        &mut self,
        rule_id: ExternalRuleId,
        options: &SmallVec<[serde_json::Value; 1]>,
    ) -> ExternalOptionsId {
        if options.is_empty() {
            ExternalOptionsId::NONE
        } else {
            self.options.push((rule_id, options.clone()))
        }
    }

    /// Send options to JS side.
    ///
    /// # Errors
    /// Returns an error if serialization of rule options fails.
    pub fn setup_rule_configs(
        &self,
        cwd: String,
        workspace_uri: Option<&str>,
        external_linter: &ExternalLinter,
    ) -> Result<(), String> {
        let json = serde_json::to_string(&ConfigSer::new(cwd, workspace_uri, self));
        match json {
            Ok(options_json) => (external_linter.setup_rule_configs)(options_json),
            Err(err) => Err(format!("Failed to serialize external plugin options: {err}")),
        }
    }
}

/// Wrapper struct for serializing options.
///
/// Splits rule IDs and options into separate arrays, without collecting into intermediate `Vec`s.
///
/// Input (`ExternalPluginStore::options`):
/// ```ignore
/// [
///   (rule_id1, [option_1a, option_1b]),
///   (rule_id2, [option_2a, option_2b]),
///   ...
/// ]
/// ```
///
/// Output JSON:
/// ```json
/// {
///   "ruleIds": [rule_id1, rule_id2, ...],
///   "options": [
///     [option_1a, option_1b],
///     [option_2a, option_2b],
///     ...
///   ],
/// }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigSer<'s> {
    cwd: String,
    workspace_uri: Option<&'s str>,
    rule_ids: ConfigSerRuleIds<'s>,
    options: ConfigSerOptions<'s>,
}

impl<'s> ConfigSer<'s> {
    fn new(
        cwd: String,
        workspace_uri: Option<&'s str>,
        external_plugin_store: &'s ExternalPluginStore,
    ) -> Self {
        Self {
            cwd,
            workspace_uri,
            rule_ids: ConfigSerRuleIds(external_plugin_store),
            options: ConfigSerOptions(external_plugin_store),
        }
    }
}

struct ConfigSerRuleIds<'s>(&'s ExternalPluginStore);

impl Serialize for ConfigSerRuleIds<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.options.iter().map(|(rule_id, _)| rule_id))
    }
}

struct ConfigSerOptions<'s>(&'s ExternalPluginStore);

impl Serialize for ConfigSerOptions<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.options.iter().map(|(_, options)| options))
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
                write!(f, "Plugin '{plugin}' not found")
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
