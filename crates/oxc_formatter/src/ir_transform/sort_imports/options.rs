use std::fmt;
use std::str::FromStr;

pub use super::group_config::{ImportModifier, ImportSelector};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortImportsOptions {
    /// Partition imports by newlines.
    /// Default is `false`.
    pub partition_by_newline: bool,
    /// Partition imports by comments.
    /// Default is `false`.
    pub partition_by_comment: bool,
    /// Sort side effects imports.
    /// Default is `false`.
    pub sort_side_effects: bool,
    /// Sort order (asc or desc).
    /// Default is ascending (asc).
    pub order: SortOrder,
    /// Ignore case when sorting.
    /// Default is `true`.
    pub ignore_case: bool,
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
    /// Each inner `Vec` represents a group, and multiple group names in the same `Vec` are treated as one.
    /// Default is defined by [`default_groups()`] function.
    pub groups: Vec<Vec<String>>,
    /// Define your own groups for matching very specific imports.
    /// Default is `[]`.
    pub custom_groups: Vec<CustomGroupDefinition>,
    /// Per-boundary newline overrides.
    /// `newline_boundary_overrides[i]` = override for boundary between `groups[i]` and `groups[i+1]`.
    /// `None` means "use global `newlines_between`".
    pub newline_boundary_overrides: Vec<Option<bool>>,
}

impl Default for SortImportsOptions {
    fn default() -> Self {
        Self {
            partition_by_newline: false,
            partition_by_comment: false,
            sort_side_effects: false,
            order: SortOrder::default(),
            ignore_case: true,
            newlines_between: true,
            internal_pattern: default_internal_patterns(),
            groups: default_groups(),
            custom_groups: vec![],
            newline_boundary_overrides: vec![],
        }
    }
}

// ---

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SortOrder {
    /// Sort in ascending order (A-Z).
    #[default]
    Asc,
    /// Sort in descending order (Z-A).
    Desc,
}

impl SortOrder {
    pub const fn is_asc(self) -> bool {
        matches!(self, Self::Asc)
    }

    pub const fn is_desc(self) -> bool {
        matches!(self, Self::Desc)
    }
}

impl FromStr for SortOrder {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err("Value not supported for SortOrder. Supported values are 'asc' and 'desc'."),
        }
    }
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct CustomGroupDefinition {
    /// The identifier used in `groups` representing this group.
    pub group_name: String,
    /// List of glob patterns to match import sources for this group.
    pub element_name_pattern: Vec<String>,
    /// When specified, the import's selectors must contain this selector.
    pub selector: Option<ImportSelector>,
    /// When specified, **all** modifiers must be present in the import's modifiers (AND logic).
    pub modifiers: Vec<ImportModifier>,
}

/// Returns default prefixes for identifying internal imports: `["~/", "@/"]`.
pub fn default_internal_patterns() -> Vec<String> {
    ["~/", "@/"].iter().map(|s| (*s).to_string()).collect()
}

/// Returns default groups configuration for organizing imports.
pub fn default_groups() -> Vec<Vec<String>> {
    vec![
        vec!["type-import".to_string()],
        vec!["value-builtin".to_string(), "value-external".to_string()],
        vec!["type-internal".to_string()],
        vec!["value-internal".to_string()],
        vec!["type-parent".to_string(), "type-sibling".to_string(), "type-index".to_string()],
        vec!["value-parent".to_string(), "value-sibling".to_string(), "value-index".to_string()],
        vec!["unknown".to_string()],
    ]
}
