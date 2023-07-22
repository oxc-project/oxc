use std::path::PathBuf;

pub type Alias = Vec<(String, Vec<AliasValue>)>;

#[derive(Debug, Clone)]
pub enum AliasValue {
    /// The path value
    Path(String),

    /// The `false` value
    Ignore,
}

#[derive(Debug, Clone)]
pub struct ResolveOptions {
    /// A list of module alias configurations or an object which maps key to value
    pub alias: Alias,

    /// A list of alias fields in description files.
    /// Specify a field, such as `browser`, to be parsed according to [this specification](https://github.com/defunctzombie/package-browser-field-spec).
    ///
    /// Default `[]`
    pub alias_fields: Vec<String>,

    /// A list of description files to read (there was once a `bower.json`).
    ///
    /// Default `["package.json"]`
    pub description_files: Vec<String>,

    /// Enforce that a extension from extensions must be used.
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

    /// A list of extensions which should be tried for.
    ///
    /// Default `[".js", ".json", ".node"]`
    pub extensions: Vec<String>,

    /// Same as [ResolveOptions::alias], Redirect module requests when normal resolving fails. .
    ///
    /// Default `[]`
    pub fallback: Alias,

    /// A list of main files in directories.
    ///
    /// Default `["index"]`
    pub main_files: Vec<String>,

    /// A list of directories to resolve modules from, can be absolute path or folder name.
    ///
    /// Default `["node_modules"]`
    pub modules: Vec<String>,

    /// prefer to resolve module requests as relative requests instead of using modules from node_modules directories.
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

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            alias: vec![],
            alias_fields: vec![],
            description_files: vec!["package.json".into()],
            enforce_extension: None,
            extension_alias: vec![],
            extensions: vec![".js".into(), ".json".into(), ".node".into()],
            fallback: vec![],
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
