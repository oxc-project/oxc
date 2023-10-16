use std::{fmt, path::PathBuf};

/// Module Resolution Options
///
/// Options are directly ported from [enhanced-resolve](https://github.com/webpack/enhanced-resolve#resolver-options).
///
/// See [webpack resolve](https://webpack.js.org/configuration/resolve/) for information and examples
#[derive(Debug, Clone)]
pub struct ResolveOptions {
    /// Path to TypeScript configuration file.
    ///
    /// Default `None`
    pub tsconfig: Option<TsconfigOptions>,

    /// Create aliases to import or require certain modules more easily.
    /// A trailing $ can also be added to the given object's keys to signify an exact match.
    pub alias: Alias,

    /// A list of alias fields in description files.
    /// Specify a field, such as `browser`, to be parsed according to [this specification](https://github.com/defunctzombie/package-browser-field-spec).
    /// Can be a path to json object such as `["path", "to", "exports"]`.
    ///
    /// Default `[]`
    pub alias_fields: Vec<Vec<String>>,

    /// Condition names for exports field which defines entry points of a package.
    /// The key order in the exports field is significant. During condition matching, earlier entries have higher priority and take precedence over later entries.
    ///
    /// Default `[]`
    pub condition_names: Vec<String>,

    /// The JSON files to use for descriptions. (There was once a `bower.json`.)
    ///
    /// Default `["package.json"]`
    pub description_files: Vec<String>,

    /// If true, it will not allow extension-less files.
    /// So by default `require('./foo')` works if `./foo` has a `.js` extension,
    /// but with this enabled only `require('./foo.js')` will work.
    ///
    /// Default to `true` when [ResolveOptions::extensions] contains an empty string.
    /// Use `Some(false)` to disable the behavior.
    /// See <https://github.com/webpack/enhanced-resolve/pull/285>
    ///
    /// Default None, which is the same as `Some(false)` when the above empty rule is not applied.
    pub enforce_extension: EnforceExtension,

    /// A list of exports fields in description files.
    /// Can be a path to json object such as `["path", "to", "exports"]`.
    ///
    /// Default `[["exports"]]`.
    pub exports_fields: Vec<Vec<String>>,

    /// An object which maps extension to extension aliases.
    ///
    /// Default `{}`
    pub extension_alias: Vec<(String, Vec<String>)>,

    /// Attempt to resolve these extensions in order.
    /// If multiple files share the same name but have different extensions,
    /// will resolve the one with the extension listed first in the array and skip the rest.
    ///
    /// Default `[".js", ".json", ".node"]`
    pub extensions: Vec<String>,

    /// Redirect module requests when normal resolving fails.
    ///
    /// Default `[]`
    pub fallback: Alias,

    /// Request passed to resolve is already fully specified and extensions or main files are not resolved for it (they are still resolved for internal requests).
    ///
    /// See also webpack configuration [resolve.fullySpecified](https://webpack.js.org/configuration/module/#resolvefullyspecified)
    ///
    /// Default `false`
    pub fully_specified: bool,

    /// A list of main fields in description files
    ///
    /// Default `["main"]`.
    pub main_fields: Vec<String>,

    /// The filename to be used while resolving directories.
    ///
    /// Default `["index"]`
    pub main_files: Vec<String>,

    /// A list of directories to resolve modules from, can be absolute path or folder name.
    ///
    /// Default `["node_modules"]`
    pub modules: Vec<String>,

    /// Resolve to a context instead of a file.
    ///
    /// Default `false`
    pub resolve_to_context: bool,

    /// Prefer to resolve module requests as relative requests instead of using modules from node_modules directories.
    ///
    /// Default `false`
    pub prefer_relative: bool,

    /// Prefer to resolve server-relative urls as absolute paths before falling back to resolve in ResolveOptions::roots.
    ///
    /// Default `false`
    pub prefer_absolute: bool,

    /// A list of resolve restrictions to restrict the paths that a request can be resolved on.
    ///
    /// Default `[]`
    pub restrictions: Vec<Restriction>,

    /// A list of directories where requests of server-relative URLs (starting with '/') are resolved.
    /// On non-Windows systems these requests are resolved as an absolute path first.
    ///
    /// Default `[]`
    pub roots: Vec<PathBuf>,

    /// Whether to resolve symlinks to their symlinked location.
    /// When enabled, symlinked resources are resolved to their real path, not their symlinked location.
    /// Note that this may cause module resolution to fail when using tools that symlink packages (like npm link).
    ///
    /// Default `true`
    pub symlinks: bool,

    /// Whether to parse [module.builtinModules](https://nodejs.org/api/module.html#modulebuiltinmodules) or not.
    /// For example, "zlib" will throw [ResolveError::Builtin] when set to true.
    ///
    /// Default `false`
    pub builtin_modules: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforceExtension {
    Auto,
    Enabled,
    Disabled,
}

impl EnforceExtension {
    pub fn is_auto(&self) -> bool {
        *self == Self::Auto
    }

    pub fn is_enabled(&self) -> bool {
        *self == Self::Enabled
    }

    pub fn is_disabled(&self) -> bool {
        *self == Self::Disabled
    }
}

/// Alias for [ResolveOptions::alias] and [ResolveOptions::fallback].
pub type Alias = Vec<(String, Vec<AliasValue>)>;

/// Alias Value for [ResolveOptions::alias] and [ResolveOptions::fallback].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AliasValue {
    /// The path value
    Path(String),

    /// The `false` value
    Ignore,
}

#[derive(Debug, Clone)]
pub enum Restriction {
    Path(PathBuf),
    RegExp(String),
}

/// Tsconfig Options
///
/// Derived from [tsconfig-paths-webpack-plugin](https://github.com/dividab/tsconfig-paths-webpack-plugin#options)
#[derive(Debug, Clone)]
pub struct TsconfigOptions {
    /// Allows you to specify where to find the TypeScript configuration file.
    /// You may provide
    /// * a relative path to the configuration file. It will be resolved relative to cwd.
    /// * an absolute path to the configuration file.
    pub config_file: PathBuf,

    /// Support for Typescript Project References.
    pub references: TsconfigReferences,
}

#[derive(Debug, Clone)]
pub enum TsconfigReferences {
    Disabled,
    /// Use the `references` field from tsconfig read from `config_file`.
    Auto,
    /// Manually provided relative or absolute path.
    Paths(Vec<PathBuf>),
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            tsconfig: None,
            alias: vec![],
            alias_fields: vec![],
            condition_names: vec![],
            description_files: vec!["package.json".into()],
            enforce_extension: EnforceExtension::Auto,
            extension_alias: vec![],
            exports_fields: vec![vec!["exports".into()]],
            extensions: vec![".js".into(), ".json".into(), ".node".into()],
            fallback: vec![],
            fully_specified: false,
            main_fields: vec!["main".into()],
            main_files: vec!["index".into()],
            modules: vec!["node_modules".into()],
            resolve_to_context: false,
            prefer_relative: false,
            prefer_absolute: false,
            restrictions: vec![],
            roots: vec![],
            symlinks: true,
            builtin_modules: false,
        }
    }
}

impl ResolveOptions {
    pub(crate) fn sanitize(mut self) -> Self {
        // Set `enforceExtension` to `true` when [ResolveOptions::extensions] contains an empty string.
        // See <https://github.com/webpack/enhanced-resolve/pull/285>
        if self.enforce_extension == EnforceExtension::Auto {
            if !self.extensions.is_empty() && self.extensions.iter().any(String::is_empty) {
                self.enforce_extension = EnforceExtension::Enabled;
            } else {
                self.enforce_extension = EnforceExtension::Disabled;
            }
        }
        self
    }
}

// For tracing
impl fmt::Display for ResolveOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(tsconfig) = &self.tsconfig {
            write!(f, "tsconfig:{tsconfig:?},")?;
        }
        if !self.alias.is_empty() {
            write!(f, "alias:{:?},", self.alias)?;
        }
        if !self.alias_fields.is_empty() {
            write!(f, "alias_fields:{:?},", self.alias_fields)?;
        }
        if !self.condition_names.is_empty() {
            write!(f, "condition_names:{:?},", self.condition_names)?;
        }
        if self.enforce_extension.is_enabled() {
            write!(f, "enforce_extension:{:?},", self.enforce_extension)?;
        }
        if !self.exports_fields.is_empty() {
            write!(f, "exports_fields:{:?},", self.exports_fields)?;
        }
        if !self.extension_alias.is_empty() {
            write!(f, "extension_alias:{:?},", self.extension_alias)?;
        }
        if !self.extensions.is_empty() {
            write!(f, "extensions:{:?},", self.extensions)?;
        }
        if !self.fallback.is_empty() {
            write!(f, "fallback:{:?},", self.fallback)?;
        }
        if self.fully_specified {
            write!(f, "fully_specified:{:?},", self.fully_specified)?;
        }
        if !self.main_fields.is_empty() {
            write!(f, "main_fields:{:?},", self.main_fields)?;
        }
        if !self.main_files.is_empty() {
            write!(f, "main_files:{:?},", self.main_files)?;
        }
        if !self.modules.is_empty() {
            write!(f, "modules:{:?},", self.modules)?;
        }
        if self.resolve_to_context {
            write!(f, "resolve_to_context:{:?},", self.resolve_to_context)?;
        }
        if self.prefer_relative {
            write!(f, "prefer_relative:{:?},", self.prefer_relative)?;
        }
        if self.prefer_absolute {
            write!(f, "prefer_absolute:{:?},", self.prefer_absolute)?;
        }
        if !self.restrictions.is_empty() {
            write!(f, "restrictions:{:?},", self.restrictions)?;
        }
        if !self.roots.is_empty() {
            write!(f, "roots:{:?},", self.roots)?;
        }
        if self.symlinks {
            write!(f, "symlinks:{:?},", self.symlinks)?;
        }
        if self.builtin_modules {
            write!(f, "builtin_modules:{:?},", self.builtin_modules)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{
        AliasValue, EnforceExtension, ResolveOptions, Restriction, TsconfigOptions,
        TsconfigReferences,
    };
    use std::path::PathBuf;

    #[test]
    fn enforce_extension() {
        assert!(EnforceExtension::Auto.is_auto());
        assert!(!EnforceExtension::Enabled.is_auto());
        assert!(!EnforceExtension::Disabled.is_auto());

        assert!(!EnforceExtension::Auto.is_enabled());
        assert!(EnforceExtension::Enabled.is_enabled());
        assert!(!EnforceExtension::Disabled.is_enabled());

        assert!(!EnforceExtension::Auto.is_disabled());
        assert!(!EnforceExtension::Enabled.is_disabled());
        assert!(EnforceExtension::Disabled.is_disabled());
    }

    #[test]
    fn display() {
        let options = ResolveOptions {
            tsconfig: Some(TsconfigOptions {
                config_file: PathBuf::from("tsconfig.json"),
                references: TsconfigReferences::Auto,
            }),
            alias: vec![("a".into(), vec![AliasValue::Ignore])],
            alias_fields: vec![vec!["browser".into()]],
            condition_names: vec!["require".into()],
            enforce_extension: EnforceExtension::Enabled,
            extension_alias: vec![(".js".into(), vec![".ts".into()])],
            exports_fields: vec![vec!["exports".into()]],
            fallback: vec![("fallback".into(), vec![AliasValue::Ignore])],
            fully_specified: true,
            resolve_to_context: true,
            prefer_relative: true,
            prefer_absolute: true,
            restrictions: vec![Restriction::Path(PathBuf::from("restrictions"))],
            roots: vec![PathBuf::from("roots")],
            builtin_modules: true,
            ..ResolveOptions::default()
        };

        let expected = r#"tsconfig:TsconfigOptions { config_file: "tsconfig.json", references: Auto },alias:[("a", [Ignore])],alias_fields:[["browser"]],condition_names:["require"],enforce_extension:Enabled,exports_fields:[["exports"]],extension_alias:[(".js", [".ts"])],extensions:[".js", ".json", ".node"],fallback:[("fallback", [Ignore])],fully_specified:true,main_fields:["main"],main_files:["index"],modules:["node_modules"],resolve_to_context:true,prefer_relative:true,prefer_absolute:true,restrictions:[Path("restrictions")],roots:["roots"],symlinks:true,builtin_modules:true,"#;
        assert_eq!(format!("{options}"), expected);
    }
}
