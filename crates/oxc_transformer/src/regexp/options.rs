use crate::env::{can_enable_plugin, Versions};

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

impl RegExpOptions {
    #[must_use]
    pub fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            sticky_flag: can_enable_plugin("transform-sticky-regex", targets, bugfixes),
            unicode_flag: can_enable_plugin("transform-unicode-regex", targets, bugfixes),
            dot_all_flag: can_enable_plugin("transform-dotall-regex", targets, bugfixes),
            look_behind_assertions: can_enable_plugin(
                "esbuild-regexp-lookbehind-assertions",
                targets,
                bugfixes,
            ),
            named_capture_groups: can_enable_plugin(
                "transform-named-capturing-groups-regex",
                targets,
                bugfixes,
            ),
            unicode_property_escapes: can_enable_plugin(
                "transform-unicode-property-regex",
                targets,
                bugfixes,
            ),
            match_indices: can_enable_plugin("esbuild-regexp-match-indices", targets, bugfixes),
            set_notation: can_enable_plugin("transform-unicode-sets-regex", targets, bugfixes),
        }
    }
}
