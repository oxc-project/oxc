pub struct Es2021Options {
    /// https://babeljs.io/docs/babel-plugin-transform-logical-assignment-operators
    pub logical_assignment_operators: bool,
    /// https://babeljs.io/docs/babel-plugin-transform-numeric-separator
    pub numeric_separators: bool,
}

impl Default for Es2021Options {
    fn default() -> Self {
        Self { logical_assignment_operators: true, numeric_separators: true }
    }
}
