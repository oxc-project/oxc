#[derive(Debug, Clone)]
pub struct ResolveOptions {
    /// A list of alias fields in description files.
    /// Specify a field, such as `browser`, to be parsed according to [this specification](https://github.com/defunctzombie/package-browser-field-spec).
    ///
    /// Default `[]`
    pub alias_fields: Vec<String>,

    /// An object which maps extension to extension aliases
    ///
    /// Default `{}`
    pub extension_alias: Vec<(String, Vec<String>)>,

    /// Enforce that a extension from extensions must be used
    ///
    /// Default `false`
    pub enforce_extension: bool,

    /// A list of extensions which should be tried for
    ///
    /// Default `[".js", ".json", ".node"]`
    pub extensions: Vec<String>,

    /// A list of main files in directories
    ///
    /// Default `["index"]`
    pub main_files: Vec<String>,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            alias_fields: vec![],
            extension_alias: vec![],
            enforce_extension: false,
            extensions: vec![".js".into(), ".json".into(), ".node".into()],
            main_files: vec!["index".into()],
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
