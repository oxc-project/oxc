//! Generic "groups" vocabulary shared by every sorting target.
//!
//! A group name is `(modifier-)*selector`, e.g. `"type-external"` for imports.
//! Selector and modifier spellings are `snake_case` and never contain `-`,
//! so the parser can split on `-` (the last part is the selector).
//! Each target supplies its own `SortVocabulary` (the enums plus their parsers).

use std::{cmp::Ordering, fmt, hash::Hash};

use rustc_hash::FxHashMap;

/// The selector/modifier enums of one sorting target.
///
/// The bounds on the marker exist only because `#[derive]` on the generic group types adds a
/// `V: Trait` bound to each generated impl; the marker itself is never stored.
pub trait SortVocabulary: Clone + fmt::Debug + Eq + Hash {
    type Selector: Copy + Clone + fmt::Debug + PartialEq + Eq + PartialOrd + Ord + Hash;
    type Modifier: Copy + Clone + fmt::Debug + PartialEq + Eq + PartialOrd + Ord + Hash;

    fn parse_selector(s: &str) -> Option<Self::Selector>;
    fn parse_modifier(s: &str) -> Option<Self::Modifier>;
}

/// A parsed entry in a `groups` configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupEntry<V: SortVocabulary> {
    /// A predefined group name (e.g. `"type-external"`).
    Predefined(GroupName<V>),
    /// The special `"unknown"` catch-all group.
    Unknown,
    /// A reference to a user-defined custom group by name.
    Custom(String),
}

impl<V: SortVocabulary> GroupEntry<V> {
    /// Parse a group entry string.
    ///
    /// - `"unknown"`: `GroupEntry::Unknown`
    /// - a valid predefined name: `GroupEntry::Predefined(..)`
    /// - anything else: `GroupEntry::Custom(..)`
    ///
    /// NOTE: Dangling custom names are not detected here; the target's options `validate` does that.
    pub fn parse(name: &str) -> Self {
        if name == "unknown" {
            return Self::Unknown;
        }
        if let Some(group_name) = GroupName::parse(name) {
            return Self::Predefined(group_name);
        }
        Self::Custom(name.to_string())
    }
}

/// A predefined group name: exactly one selector and any number of modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupName<V: SortVocabulary> {
    pub selector: V::Selector,
    pub modifiers: Vec<V::Modifier>,
}

impl<V: SortVocabulary> GroupName<V> {
    /// A selector with no modifiers.
    pub fn is_plain_selector(&self, selector: V::Selector) -> bool {
        self.selector == selector && self.modifiers.is_empty()
    }

    /// Parse `(modifier-)*selector`.
    ///
    /// Modifier order is normalized (sorted, deduplicated) so `"type-value-x"` and `"value-type-x"`
    /// are the same name.
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        let selector = V::parse_selector(parts.last()?)?;

        if parts.len() == 1 {
            return Some(Self { modifiers: vec![], selector });
        }

        let mut modifiers = Vec::with_capacity(parts.len() - 1);
        for part in &parts[..parts.len() - 1] {
            modifiers.push(V::parse_modifier(part)?);
        }
        modifiers.sort_unstable();
        modifiers.dedup();

        Some(Self { selector, modifiers })
    }

    /// Whether this name can describe an element with the given selectors and modifiers.
    pub fn is_a_possible_name_of(
        &self,
        selectors: &[V::Selector],
        modifiers: &[V::Modifier],
    ) -> bool {
        selectors.contains(&self.selector) && self.modifiers.iter().all(|m| modifiers.contains(m))
    }
}

impl<V: SortVocabulary> PartialOrd for GroupName<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Ordering by specificity: selector priority first (the enum's declaration order),
/// then MORE modifiers first, then the modifiers themselves.
/// `GroupMatcher` scans predefined names in this order so the most specific match wins.
impl<V: SortVocabulary> Ord for GroupName<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.selector.cmp(&other.selector) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match other.modifiers.len().cmp(&self.modifiers.len()) {
            Ordering::Equal => self.modifiers.cmp(&other.modifiers),
            ord => ord,
        }
    }
}

/// A user-defined group. All set conditions must hold (AND).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomGroup<V: SortVocabulary> {
    /// The identifier used in `groups` to reference this group.
    pub group_name: String,
    /// Glob patterns the element name must match (any of them; empty = no name condition).
    pub element_name_pattern: Vec<String>,
    /// When set, the element's selectors must contain it.
    pub selector: Option<V::Selector>,
    /// When non-empty, the element's modifiers must contain all of them.
    pub modifiers: Vec<V::Modifier>,
}

impl<V: SortVocabulary> Default for CustomGroup<V> {
    fn default() -> Self {
        Self {
            group_name: String::new(),
            element_name_pattern: vec![],
            selector: None,
            modifiers: vec![],
        }
    }
}

impl<V: SortVocabulary> CustomGroup<V> {
    /// Check if this is a plain selector: the given selector with no other narrowing condition.
    /// The custom-group counterpart of [`GroupName::is_plain_selector`];
    /// keep in sync when a narrowing field is added to this struct.
    pub fn is_plain_selector(&self, selector: V::Selector) -> bool {
        self.selector == Some(selector)
            && self.element_name_pattern.is_empty()
            && self.modifiers.is_empty()
    }
}

/// Resolves an element to the index of its group in the `groups` list.
pub struct GroupMatcher<V: SortVocabulary> {
    /// Custom groups referenced from `groups`, in `custom_groups` order (first match wins).
    custom_groups: Vec<(CustomGroup<V>, usize)>,
    /// Predefined names sorted most-specific first, so the first possible name is the best one.
    predefined_groups: Vec<(GroupName<V>, usize)>,
    /// Index of `"unknown"` in `groups`, or `groups.len()` when absent.
    unknown_group_index: usize,
}

impl<V: SortVocabulary> GroupMatcher<V> {
    pub fn new(groups: &[Vec<GroupEntry<V>>], custom_groups: &[CustomGroup<V>]) -> Self {
        let mut unknown_group_index: Option<usize> = None;
        let mut used_custom_group_index_map = FxHashMap::default();
        let mut predefined_groups = Vec::new();

        for (index, group_union) in groups.iter().enumerate() {
            for entry in group_union {
                match entry {
                    GroupEntry::Unknown => unknown_group_index = Some(index),
                    GroupEntry::Custom(name) => {
                        used_custom_group_index_map.insert(name.as_str(), index);
                    }
                    GroupEntry::Predefined(group_name) => {
                        predefined_groups.push((group_name.clone(), index));
                    }
                }
            }
        }

        let mut used_custom_groups = Vec::with_capacity(used_custom_group_index_map.len());
        for custom_group in custom_groups {
            if let Some(index) = used_custom_group_index_map.get(custom_group.group_name.as_str()) {
                used_custom_groups.push((custom_group.clone(), *index));
            }
        }

        predefined_groups.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            custom_groups: used_custom_groups,
            predefined_groups,
            unknown_group_index: unknown_group_index.unwrap_or(groups.len()),
        }
    }

    /// Custom groups first (in definition order), then the most specific predefined name,
    /// then `unknown`.
    pub fn group_index(
        &self,
        name: &str,
        selectors: &[V::Selector],
        modifiers: &[V::Modifier],
    ) -> usize {
        for (custom_group, index) in &self.custom_groups {
            let name_matches = custom_group.element_name_pattern.is_empty()
                || custom_group
                    .element_name_pattern
                    .iter()
                    .any(|pattern| fast_glob::glob_match(pattern, name));
            let selector_matches = custom_group.selector.is_none_or(|s| selectors.contains(&s));
            let modifiers_match = custom_group.modifiers.iter().all(|m| modifiers.contains(m));
            if name_matches && selector_matches && modifiers_match {
                return *index;
            }
        }

        for (group_name, index) in &self.predefined_groups {
            if group_name.is_a_possible_name_of(selectors, modifiers) {
                return *index;
            }
        }

        self.unknown_group_index
    }

    /// Whether `groups` uses `selector` on its own (no other narrowing condition):
    /// either the predefined name, or a referenced custom group with the bare selector.
    /// A custom group that merely happens to match (by pattern or modifiers) does not count.
    pub fn has_plain_selector(&self, selector: V::Selector) -> bool {
        self.predefined_groups.iter().any(|(group, _)| group.is_plain_selector(selector))
            || self.custom_groups.iter().any(|(group, _)| group.is_plain_selector(selector))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct TestVocab;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum Sel {
        Method,
        Property,
        Member,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum Mod {
        Static,
        Optional,
    }

    impl SortVocabulary for TestVocab {
        type Selector = Sel;
        type Modifier = Mod;

        fn parse_selector(s: &str) -> Option<Sel> {
            match s {
                "method" => Some(Sel::Method),
                "property" => Some(Sel::Property),
                "member" => Some(Sel::Member),
                _ => None,
            }
        }

        fn parse_modifier(s: &str) -> Option<Mod> {
            match s {
                "static" => Some(Mod::Static),
                "optional" => Some(Mod::Optional),
                _ => None,
            }
        }
    }

    type Name = GroupName<TestVocab>;
    type Entry = GroupEntry<TestVocab>;
    type Custom = CustomGroup<TestVocab>;

    #[test]
    fn parses_modifiers_then_selector() {
        let name = Name::parse("optional-static-method").unwrap();
        assert_eq!(name.selector, Sel::Method);
        // Normalized: sorted by the modifier enum's `Ord`, duplicates removed.
        assert_eq!(name.modifiers, vec![Mod::Static, Mod::Optional]);
        assert_eq!(Name::parse("static-static-method").unwrap().modifiers, vec![Mod::Static]);
    }

    #[test]
    fn rejects_unknown_parts_and_missing_selector() {
        assert!(Name::parse("foo-method").is_none());
        assert!(Name::parse("static").is_none());
        assert!(Name::parse("").is_none());
    }

    #[test]
    fn group_entry_parse_classifies() {
        assert_eq!(Entry::parse("unknown"), Entry::Unknown);
        assert!(matches!(Entry::parse("static-method"), Entry::Predefined(_)));
        assert_eq!(Entry::parse("mine"), Entry::Custom("mine".to_string()));
    }

    #[test]
    fn more_specific_names_sort_first() {
        let plain = Name::parse("method").unwrap();
        let specific = Name::parse("static-method").unwrap();
        assert!(specific < plain);
    }

    #[test]
    fn selector_priority_beats_modifier_count() {
        // Selector order wins over modifier count: `property` (no modifiers) still sorts after
        // `static-method`, because `Property > Method` regardless of modifiers.
        assert!(Name::parse("property").unwrap() > Name::parse("static-method").unwrap());
    }

    #[test]
    fn matcher_prefers_custom_then_most_specific_predefined() {
        let groups = vec![
            vec![Entry::Custom("mine".to_string())],
            vec![Entry::parse("static-method")],
            vec![Entry::parse("method")],
            vec![Entry::Unknown],
        ];
        let custom = vec![Custom {
            group_name: "mine".to_string(),
            element_name_pattern: vec!["foo*".to_string()],
            selector: None,
            modifiers: vec![],
        }];
        let matcher = GroupMatcher::new(&groups, &custom);

        assert_eq!(matcher.group_index("fooBar", &[Sel::Method, Sel::Member], &[Mod::Static]), 0);
        assert_eq!(matcher.group_index("bar", &[Sel::Method, Sel::Member], &[Mod::Static]), 1);
        assert_eq!(matcher.group_index("bar", &[Sel::Method, Sel::Member], &[]), 2);
        assert_eq!(matcher.group_index("x", &[Sel::Property, Sel::Member], &[]), 3);
    }

    #[test]
    fn unknown_defaults_to_groups_len_when_absent() {
        let groups = vec![vec![Entry::parse("method")]];
        let matcher = GroupMatcher::<TestVocab>::new(&groups, &[]);
        assert_eq!(matcher.group_index("x", &[Sel::Property, Sel::Member], &[]), 1);
        assert!(matcher.has_plain_selector(Sel::Method));
        assert!(!matcher.has_plain_selector(Sel::Property));
    }

    #[test]
    fn custom_group_conditions_are_anded() {
        let groups = vec![vec![Entry::Custom("opt".to_string())], vec![Entry::Unknown]];
        let custom = vec![Custom {
            group_name: "opt".to_string(),
            element_name_pattern: vec![],
            selector: Some(Sel::Property),
            modifiers: vec![Mod::Optional],
        }];
        let matcher = GroupMatcher::new(&groups, &custom);
        assert_eq!(matcher.group_index("a", &[Sel::Property, Sel::Member], &[Mod::Optional]), 0);
        assert_eq!(matcher.group_index("a", &[Sel::Property, Sel::Member], &[]), 1);
        assert_eq!(matcher.group_index("a", &[Sel::Method, Sel::Member], &[Mod::Optional]), 1);
    }

    #[test]
    fn custom_groups_not_referenced_in_groups_are_ignored() {
        let groups = vec![vec![Entry::Unknown]];
        let custom = vec![Custom { group_name: "unused".to_string(), ..Custom::default() }];
        let matcher = GroupMatcher::new(&groups, &custom);
        assert_eq!(matcher.group_index("anything", &[Sel::Member], &[]), 0);
    }

    #[test]
    fn duplicate_custom_name_in_groups_uses_last_index() {
        let groups = vec![
            vec![Entry::Custom("mine".to_string())],
            vec![Entry::Unknown],
            vec![Entry::Custom("mine".to_string())],
        ];
        let custom = vec![Custom {
            group_name: "mine".to_string(),
            element_name_pattern: vec!["x*".to_string()],
            selector: None,
            modifiers: vec![],
        }];
        let matcher = GroupMatcher::new(&groups, &custom);
        assert_eq!(matcher.group_index("x1", &[Sel::Member], &[]), 2);
    }
}
