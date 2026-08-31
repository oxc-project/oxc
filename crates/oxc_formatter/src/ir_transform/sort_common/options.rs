//! Option enums shared by every sorting target.

use std::{fmt, str::FromStr};

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

/// The primary comparison.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SortType {
    /// Natural order: digit runs compare numerically (`a2` < `a10`). The default.
    #[default]
    Natural,
    /// Code-point order of the normalized key. No locale collation.
    Alphabetical,
    /// Shorter members first (by the member's printed width).
    LineLength,
    /// Do not sort; only group (with `groups`) and partition.
    Unsorted,
}

impl SortType {
    pub const ALL: &[SortType] =
        &[SortType::Natural, SortType::Alphabetical, SortType::LineLength, SortType::Unsorted];

    pub fn name(self) -> &'static str {
        match self {
            SortType::Natural => "natural",
            SortType::Alphabetical => "alphabetical",
            SortType::LineLength => "line-length",
            SortType::Unsorted => "unsorted",
        }
    }
}

impl FromStr for SortType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "natural" => Ok(Self::Natural),
            "alphabetical" => Ok(Self::Alphabetical),
            "line-length" => Ok(Self::LineLength),
            "unsorted" => Ok(Self::Unsorted),
            _ => Err(
                "Value not supported for SortType. Supported values are 'natural', 'alphabetical', 'line-length' and 'unsorted'.",
            ),
        }
    }
}

/// What to do with non-letter characters before comparing.
///
/// "Letter" is ASCII `a-z`/`A-Z` plus Latin-1 Supplement / Latin Extended-A/B
/// (`U+00C0..=U+024F`) and Latin Extended Additional (`U+1E00..=U+1EFF`). Digits are NOT letters,
/// so `remove` strips them too; this mirrors perfectionist so configurations port unchanged.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SpecialCharacters {
    /// Compare the key as-is. The default.
    #[default]
    Keep,
    /// Strip leading non-letters (`./foo` → `foo`, `@scope/x` → `scope/x`).
    Trim,
    /// Strip every non-letter.
    Remove,
}

impl SpecialCharacters {
    pub const ALL: &[SpecialCharacters] =
        &[SpecialCharacters::Keep, SpecialCharacters::Trim, SpecialCharacters::Remove];

    pub fn name(self) -> &'static str {
        match self {
            SpecialCharacters::Keep => "keep",
            SpecialCharacters::Trim => "trim",
            SpecialCharacters::Remove => "remove",
        }
    }
}

impl FromStr for SpecialCharacters {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "keep" => Ok(Self::Keep),
            "trim" => Ok(Self::Trim),
            "remove" => Ok(Self::Remove),
            _ => Err(
                "Value not supported for SpecialCharacters. Supported values are 'keep', 'trim' and 'remove'.",
            ),
        }
    }
}

/// Secondary comparison, applied only when the primary one is `Equal`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FallbackSort {
    pub sort_type: SortType,
    /// `None` inherits the primary `order`.
    pub order: Option<SortOrder>,
}

impl Default for FallbackSort {
    fn default() -> Self {
        Self { sort_type: SortType::Unsorted, order: None }
    }
}

/// Options every sorting target shares. Targets embed this as `common`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SortCommonOptions {
    /// Default `Natural`.
    pub sort_type: SortType,
    /// Default `Asc`.
    pub order: SortOrder,
    /// Default `true`.
    pub ignore_case: bool,
    /// Default `Keep`.
    pub special_characters: SpecialCharacters,
    /// Default `{ Unsorted, None }` (equal members keep source order).
    pub fallback_sort: FallbackSort,
    /// Blank lines split the list into independently sorted partitions. Default `false`.
    pub partition_by_newline: bool,
    /// Own-line comments split the list into independently sorted partitions. Default `false`.
    pub partition_by_comment: bool,
}

impl Default for SortCommonOptions {
    fn default() -> Self {
        Self {
            sort_type: SortType::Natural,
            order: SortOrder::Asc,
            ignore_case: true,
            special_characters: SpecialCharacters::Keep,
            fallback_sort: FallbackSort::default(),
            partition_by_newline: false,
            partition_by_comment: false,
        }
    }
}
