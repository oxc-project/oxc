pub use super::group_config::{
    GroupEntry, GroupName, ImportModifier, ImportSelector, ImportVocabulary,
};
use crate::ir_transform::sort_common::options::SortCommonOptions;
pub use crate::ir_transform::sort_common::options::SortOrder;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortImportsOptions {
    /// Options shared by every sorting target (`type`, `order`, `ignoreCase`, `specialCharacters`,
    /// `fallbackSort`, `partitionByNewline`, `partitionByComment`).
    pub common: SortCommonOptions,
    /// Sort side effects imports.
    /// Default is `false`.
    pub sort_side_effects: bool,
    /// Whether to insert blank lines between different import groups.
    /// - `true`: Insert one blank line between groups (default)
    /// - `false`: No blank lines between groups
    ///
    /// NOTE: Cannot be used together with `partition_by_newline: true`.
    pub newlines_between: bool,
    /// Prefixes for internal imports.
    /// Defaults to `["~/", "@/"]`.
    pub internal_pattern: Vec<String>,
    /// Groups configuration for organizing imports.
    /// Each inner `Vec` represents a group, and multiple entries in the same `Vec` are treated as one.
    /// Default is defined by [`default_groups()`] function.
    pub groups: Vec<Vec<GroupEntry>>,
    /// Define your own groups for matching very specific imports.
    /// Default is `[]`.
    pub custom_groups: Vec<CustomGroupDefinition>,
    /// Per-boundary newline overrides.
    /// `newline_boundary_overrides[i]` = override for boundary between `groups[i]` and `groups[i+1]`.
    /// `None` means "use global `newlines_between`".
    /// Either empty (no overrides anywhere) or exactly `groups.len() - 1` entries
    /// ([`Self::validate`] checks this pairing).
    pub newline_boundary_overrides: Vec<Option<bool>>,
}

impl SortImportsOptions {
    /// Validate option combinations and cross-field references.
    ///
    /// # Errors
    /// Returns an error message if incompatible options are set,
    /// or `groups` references an undefined custom group name.
    pub fn validate(&self) -> Result<(), String> {
        if self.common.partition_by_newline
            && self.newline_boundary_overrides.iter().any(Option::is_some)
        {
            return Err("`partitionByNewline` and per-group `{ \"newlinesBetween\" }` markers cannot be used together".to_string());
        }
        if self.common.partition_by_newline && self.newlines_between {
            return Err(
                "`partitionByNewline: true` and `newlinesBetween: true` cannot be used together"
                    .to_string(),
            );
        }
        // A dangling name would silently match nothing at runtime.
        for entry in self.groups.iter().flatten() {
            if let GroupEntry::Custom(name) = entry
                && !self.custom_groups.iter().any(|g| g.group_name == *name)
            {
                return Err(format!("unknown group name `{name}` in `groups`"));
            }
        }
        // A `groupName` that `GroupEntry::parse` resolves to anything but `Custom`
        // (a predefined name or `unknown`) would be silently unreachable from `groups`.
        // Checked for every definition, not just referenced ones.
        for custom_group in &self.custom_groups {
            let name = custom_group.group_name.as_str();
            if !matches!(GroupEntry::parse(name), GroupEntry::Custom(_)) {
                return Err(format!(
                    "`customGroups` name `{name}` conflicts with a predefined group name; predefined names and `unknown` cannot be used as `groupName`"
                ));
            }
        }
        // A short/long vec would silently fall back to the global setting for tail boundaries.
        if !self.newline_boundary_overrides.is_empty()
            && self.newline_boundary_overrides.len() + 1 != self.groups.len()
        {
            return Err(format!(
                "`newline_boundary_overrides` must be empty or hold exactly one entry per group boundary ({} groups need {}, got {})",
                self.groups.len(),
                self.groups.len().saturating_sub(1),
                self.newline_boundary_overrides.len()
            ));
        }
        Ok(())
    }
}

impl Default for SortImportsOptions {
    fn default() -> Self {
        Self {
            common: SortCommonOptions::default(),
            sort_side_effects: false,
            newlines_between: true,
            internal_pattern: default_internal_patterns(),
            groups: default_groups(),
            custom_groups: vec![],
            newline_boundary_overrides: vec![],
        }
    }
}

/// A user-defined import group; the matching rules are documented on `CustomGroup`.
pub type CustomGroupDefinition =
    crate::ir_transform::sort_common::groups::CustomGroup<ImportVocabulary>;

/// Returns default prefixes for identifying internal imports: `["~/", "@/", "#"]`.
pub fn default_internal_patterns() -> Vec<String> {
    ["~/", "@/", "#"].iter().map(|s| (*s).to_string()).collect()
}

/// Returns default groups configuration for organizing imports.
///
/// # Panics
///
/// Never panics in practice; the predefined group names are hard-coded and known to be valid.
pub fn default_groups() -> Vec<Vec<GroupEntry>> {
    // Helper to parse a predefined group name
    let p = |s: &str| GroupEntry::Predefined(GroupName::parse(s).unwrap());
    // Our policy: far to near, built-in to local.
    // Do not include side effects by default, it may break some code if moved around.
    vec![
        vec![p("builtin")],
        vec![p("external")],
        vec![p("internal"), p("subpath")],
        vec![p("parent"), p("sibling"), p("index")],
        vec![p("style")],
        vec![GroupEntry::Unknown],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options_with_custom_group_name(name: &str) -> SortImportsOptions {
        SortImportsOptions {
            custom_groups: vec![CustomGroupDefinition {
                group_name: name.to_string(),
                ..CustomGroupDefinition::default()
            }],
            ..SortImportsOptions::default()
        }
    }

    #[test]
    fn rejects_custom_group_names_conflicting_with_predefined() {
        // Plain predefined names, compound predefined names, and `unknown` are all reserved
        for name in
            ["side_effect", "side_effect_style", "external", "side_effect-import", "unknown"]
        {
            let err = options_with_custom_group_name(name)
                .validate()
                .expect_err("conflicting name must be rejected");
            assert!(err.contains(name), "error must name the conflicting group: {err}");
        }
    }

    #[test]
    fn accepts_non_predefined_custom_group_names() {
        // `-` is not usable in predefined names (only as their separator),
        // so `side-effect` / `side-effect-style` are valid custom names.
        for name in ["side-effect", "side-effect-style", "warp-drive", "myGroup"] {
            options_with_custom_group_name(name).validate().unwrap_or_else(|err| {
                panic!("`{name}` must be accepted as a custom group name: {err}")
            });
        }
    }
}
