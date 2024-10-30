#[derive(Default, Debug, Clone, Copy)]
pub struct RegExpOptions {
    /// Enables plugin to transform the RegExp literal has `y` flag
    pub sticky_flag: bool,

    /// Enables plugin to transform the RegExp literal has `u` flag
    pub unicode_flag: bool,

    /// Enables plugin to transform the RegExp literal has `s` flag
    pub dot_all_flag: bool,

    /// Enables plugin to transform the RegExp literal has `(?<=)` or `(?<!)` lookbehind assertions
    pub look_behind_assertions: bool,

    /// Enables plugin to transform the RegExp literal has `(?<name>x)` named capture groups
    pub named_capture_groups: bool,

    /// Enables plugin to transform the RegExp literal has `\p{}` and `\P{}` unicode property escapes
    pub unicode_property_escapes: bool,

    /// Enables plugin to transform `d` flag
    pub match_indices: bool,

    /// Enables plugin to transform the RegExp literal has `v` flag
    pub set_notation: bool,
}
