#[derive(Debug, Clone)]
pub struct ResolveOptions {
    /// A list of extensions which should be tried for
    /// Default: `[".js", ".json", ".node"]`
    pub extensions: Vec<String>,

    /// Enforce that a extension from extensions must be used
    /// Default: false
    pub enforce_extension: bool,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            extensions: vec![".js".into(), ".json".into(), ".node".into()],
            enforce_extension: false,
        }
    }
}

impl ResolveOptions {
    pub(crate) fn sanitize(mut self) -> Self {
        // Remove the leading `.` because `Path::with_extension` does not accept the leading dot.
        self.extensions = self
            .extensions
            .into_iter()
            .map(|ext| ext.trim_start_matches('.').to_string())
            .collect();
        self
    }
}
