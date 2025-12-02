use std::fmt;
use std::str::FromStr;

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
    /// If `None`, uses the default internal patterns.
    pub internal_pattern: Option<Vec<String>>,
    /// Groups configuration for organizing imports.
    /// Each inner `Vec` represents a group, and multiple group names in the same `Vec` are treated as one.
    /// If `None`, uses the default groups.
    pub groups: Option<Vec<Vec<String>>>,
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
            internal_pattern: None,
            groups: None,
        }
    }
}

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
