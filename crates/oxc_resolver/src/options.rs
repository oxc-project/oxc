use std::path::PathBuf;

/// Module Resolution Options
///
/// Options are directly ported from [enhanced-resolve](https://github.com/webpack/enhanced-resolve#resolver-options).
///
/// See [webpack resolve](https://webpack.js.org/configuration/resolve/) for information and examples
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct ResolveOptions {
    /// Create aliases to import or require certain modules more easily.
    /// A trailing $ can also be added to the given object's keys to signify an exact match.
    pub alias: Alias,

    /// A list of alias fields in description files.
    /// Specify a field, such as `browser`, to be parsed according to [this specification](https://github.com/defunctzombie/package-browser-field-spec).
    ///
    /// Default `[]`
    pub alias_fields: Vec<String>,

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
    pub enforce_extension: Option<bool>,

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

    /// The filename to be used while resolving directories.
    ///
    /// Default `["index"]`
    pub main_files: Vec<String>,

    /// A list of directories to resolve modules from, can be absolute path or folder name.
    ///
    /// Default `["node_modules"]`
    pub modules: Vec<String>,

    /// Prefer to resolve module requests as relative requests instead of using modules from node_modules directories.
    ///
    /// Default `false`
    pub prefer_relative: bool,

    /// Prefer to resolve server-relative urls as absolute paths before falling back to resolve in ResolveOptions::roots.
    ///
    /// Default `false`
    pub prefer_absolute: bool,

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
}

/// Alias for [ResolveOptions::alias] and [ResolveOptions::fallback].
pub type Alias = Vec<(String, Vec<AliasValue>)>;

/// Alias Value for [ResolveOptions::alias] and [ResolveOptions::fallback].
#[derive(Debug, Clone)]
pub enum AliasValue {
    /// The path value
    Path(String),

    /// The `false` value
    Ignore,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            alias: vec![],
            alias_fields: vec![],
            condition_names: vec![],
            description_files: vec!["package.json".into()],
            enforce_extension: None,
            extension_alias: vec![],
            extensions: vec![".js".into(), ".json".into(), ".node".into()],
            fallback: vec![],
            fully_specified: false,
            main_files: vec!["index".into()],
            modules: vec!["node_modules".into()],
            prefer_relative: false,
            prefer_absolute: false,
            roots: vec![],
            symlinks: true,
        }
    }
}

impl ResolveOptions {
    pub(crate) fn sanitize(mut self) -> Self {
        if self.enforce_extension.is_none() {
            self.enforce_extension = Some(false);
            // Set `enforceExtension` to `true` when [ResolveOptions::extensions] contains an empty string.
            // See <https://github.com/webpack/enhanced-resolve/pull/285>
            if self.extensions.iter().any(String::is_empty) {
                self.enforce_extension = Some(true);
                self.extensions.retain(String::is_empty);
            }
        }
        self.extensions = Self::remove_leading_dots(self.extensions);
        self.extension_alias = self
            .extension_alias
            .into_iter()
            .map(|(extension, extensions)| {
                (Self::remove_leading_dot(&extension), Self::remove_leading_dots(extensions))
            })
            .collect();
        self
    }

    // Remove the leading `.` because `Path::with_extension` does not accept the dot.
    fn remove_leading_dot(s: &str) -> String {
        s.trim_start_matches('.').to_string()
    }

    fn remove_leading_dots(v: Vec<String>) -> Vec<String> {
        v.into_iter().map(|s| Self::remove_leading_dot(&s)).collect()
    }
}
