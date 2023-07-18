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
    pub alias: Vec<(String, Vec<AliasValue>)>,

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
    /// Default `false`
    pub enforce_extension: bool,

    /// An object which maps extension to extension aliases.
    ///
    /// Default `{}`
    pub extension_alias: Vec<(String, Vec<String>)>,

    /// A list of extensions which should be tried for.
    ///
    /// Default `[".js", ".json", ".node"]`
    pub extensions: Vec<String>,

    /// A list of main files in directories.
    ///
    /// Default `["index"]`
    pub main_files: Vec<String>,

    /// A list of directories to resolve modules from, can be absolute path or folder name.
    ///
    /// Default `["node_modules"]`
    pub modules: Vec<String>,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            alias: vec![],
            alias_fields: vec![],
            description_files: vec!["package.json".into()],
            enforce_extension: false,
            extension_alias: vec![],
            extensions: vec![".js".into(), ".json".into(), ".node".into()],
            main_files: vec!["index".into()],
            modules: vec!["node_modules".into()],
        }
    }
}

impl ResolveOptions {
    pub(crate) fn sanitize(mut self) -> Self {
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
