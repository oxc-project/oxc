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

/// Identity of the plugin a config asks to load: which entry point of which package, at which
/// version, under which name.
///
/// The same package at the same version can be installed at several paths: npm, and Yarn with the
/// `node_modules` linker, nest a copy of a package under each dependent which cannot share a
/// hoisted one, and pnpm gives each peer dependency variant of a version its own store entry.
/// Each copy has its own realpath, so deduplication by path does not recognise them as one
/// plugin. The identity is read from the plugin's own `package.json`, which every copy carries
/// unchanged, so all copies of one package share one identity.
///
/// The alias is part of the identity because the name a plugin is registered under depends on it:
/// without an alias the plugin names itself, via `meta.name` or its package name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalPluginIdentity {
    key: ExternalPluginKey,
    /// `version` field of the plugin package's `package.json`.
    version: String,
}

impl ExternalPluginIdentity {
    pub fn new(
        package_name: &str,
        version: &str,
        entry_path: PathBuf,
        alias: Option<&str>,
    ) -> Self {
        Self {
            key: ExternalPluginKey {
                package_name: package_name.to_string(),
                entry_path,
                alias: alias.map(ToString::to_string),
            },
            version: version.to_string(),
        }
    }

    /// `name` field of the plugin package's `package.json`.
    pub fn package_name(&self) -> &str {
        &self.key.package_name
    }

    /// `version` field of the plugin package's `package.json`.
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Part of [`ExternalPluginIdentity`] which determines the name the plugin is registered under.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ExternalPluginKey {
    /// `name` field of the plugin package's `package.json`.
    package_name: String,
    /// Path of the plugin's entry point, relative to the directory containing that `package.json`.
    /// A package can contain several plugins, which all share its `package.json`.
    entry_path: PathBuf,
    /// Alias the config gives the plugin, if any.
    alias: Option<String>,
}

/// Registered installation of a plugin package.
#[derive(Debug)]
struct RegisteredPluginPackage {
    version: String,
    /// Resolved path of the plugin's entry point.
    path: PathBuf,
}

/// Two installations of the same plugin entry point, requested under the same name,
/// have different versions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalPluginIdentityConflict {
    pub registered_identity: ExternalPluginIdentity,
    pub registered_path: PathBuf,
    pub requested_identity: ExternalPluginIdentity,
    pub requested_path: PathBuf,
}

/// Store of the external (JS) plugins loaded for one workspace, and of their rules and options.
///
/// # Semantics
///
/// Deduplication of plugins:
///
/// 1. A plugin whose resolved path is already registered is reused
///    ([`ExternalPluginStore::is_plugin_registered`]).
/// 2. A plugin whose resolved path is new is reused when a plugin with the same
///    [`ExternalPluginIdentity`] is registered. The JS module is not imported a second time.
/// 3. Two requests with different aliases have different identities. The plugin is loaded and
///    registered under each name.
/// 4. When a plugin with the same package, entry point and alias is registered at a different
///    version, [`ExternalPluginStore::try_reuse_plugin`] returns an
///    [`ExternalPluginIdentityConflict`] and the plugin is not loaded.
/// 5. A `package.json` providing no `name` or no `version` yields no identity. The plugin is then
///    loaded, and a name clash is reported by the JS side as before.
/// 6. Deduplication is scoped to one store. The language server creates one store per workspace
///    folder, matching the per-workspace plugin registry on JS side.
/// 7. Rules of a reused plugin keep the [`ExternalRuleId`]s of the first registration.
#[derive(Debug)]
pub struct ExternalPluginStore {
    registered_plugin_paths: FxHashSet<PathBuf>,
    /// Installations of plugin packages registered so far, keyed by the part of their identity
    /// which determines the name they are registered under.
    registered_plugin_packages: FxHashMap<ExternalPluginKey, RegisteredPluginPackage>,

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
            registered_plugin_packages: FxHashMap::default(),
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

    /// Reuse an already registered plugin for the plugin resolved at `plugin_path`, if it is
    /// another installation of a plugin already registered.
    ///
    /// Returns `true` if the plugin is reused. The caller must then not load the plugin on JS side:
    /// importing the module a second time would ask the JS side to register its rules under a name
    /// which is already taken, which it rejects.
    ///
    /// Returns `false` when `identity` is `None`, or no plugin with the same package, entry point
    /// and alias is registered. See [`ExternalPluginStore`] for the full semantics.
    ///
    /// `plugin_path` is recorded when the plugin is reused, so a later config resolving to the same
    /// path is matched by [`ExternalPluginStore::is_plugin_registered`].
    ///
    /// # Errors
    /// Returns [`ExternalPluginIdentityConflict`] if a plugin with the same package, entry point
    /// and alias is registered at a different version.
    pub fn try_reuse_plugin(
        &mut self,
        plugin_path: &Path,
        identity: Option<&ExternalPluginIdentity>,
    ) -> Result<bool, ExternalPluginIdentityConflict> {
        let Some(identity) = identity else { return Ok(false) };
        let Some(registered) = self.registered_plugin_packages.get(&identity.key) else {
            return Ok(false);
        };

        if registered.version != identity.version {
            return Err(ExternalPluginIdentityConflict {
                registered_identity: ExternalPluginIdentity {
                    key: identity.key.clone(),
                    version: registered.version.clone(),
                },
                registered_path: registered.path.clone(),
                requested_identity: identity.clone(),
                requested_path: plugin_path.to_path_buf(),
            });
        }

        self.registered_plugin_paths.insert(plugin_path.to_path_buf());
        Ok(true)
    }

    /// Register plugin.
    ///
    /// `identity` is `None` when the plugin's `package.json` does not provide the fields an
    /// identity is made of. The plugin then takes part in deduplication by path only.
    ///
    /// # Panics
    /// Panics if:
    /// - Plugin at `plugin_path` is already registered.
    /// - `offset` does not equal the number of registered rules.
    pub fn register_plugin(
        &mut self,
        plugin_path: PathBuf,
        plugin_name: String,
        identity: Option<ExternalPluginIdentity>,
        offset: usize,
        rule_names: Vec<String>,
    ) {
        let newly_inserted = self.registered_plugin_paths.insert(plugin_path.clone());
        assert!(newly_inserted, "register_plugin: plugin already registered");

        if let Some(identity) = identity {
            self.registered_plugin_packages.insert(
                identity.key,
                RegisteredPluginPackage { version: identity.version, path: plugin_path },
            );
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
