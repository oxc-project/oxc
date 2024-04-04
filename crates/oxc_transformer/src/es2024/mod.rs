pub struct Es2024Options {
    /// https://babeljs.io/docs/babel-plugin-transform-unicode-sets-regex
    pub unicode_sets_regex: bool,
}

impl Default for Es2024Options {
    fn default() -> Self {
        Self { unicode_sets_regex: true }
    }
}
