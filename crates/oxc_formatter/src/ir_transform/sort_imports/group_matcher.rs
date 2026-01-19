use lazy_regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};

use super::group_config::{GroupName, ImportModifier, ImportSelector};
use super::options::CustomGroupDefinition;

// Intermediate import metadata that is used for group matching
pub struct ImportMetadata<'a> {
    pub source: &'a str,
    pub selectors: Vec<ImportSelector>,
    pub modifiers: Vec<ImportModifier>,
}

pub struct GroupMatcher {
    // Custom groups matching regex patterns
    custom_groups: Vec<(Vec<Regex>, usize)>,

    // Predefined groups sorted by priority,
    // so that we don't need to enumerate all possible group names of a given import.
    predefined_groups: Vec<(GroupName, usize)>,

    // The index of "unknown" in groups or `groups.len()` if absent
    unknown_group_index: usize,
}

impl GroupMatcher {
    pub fn new(groups: &[Vec<String>], custom_groups: &[CustomGroupDefinition]) -> Self {
        let custom_group_name_set =
            custom_groups.iter().map(|g| g.group_name.clone()).collect::<FxHashSet<_>>();

        let mut unknown_group_index: Option<usize> = None;

        let mut used_custom_group_index_map = FxHashMap::default();
        let mut predefined_groups = Vec::new();
        for (index, group_union) in groups.iter().enumerate() {
            for group in group_union {
                if group == "unknown" {
                    unknown_group_index = Some(index);
                } else if custom_group_name_set.contains(group) {
                    used_custom_group_index_map.insert(group.to_owned(), index);
                } else if let Some(group_name) = GroupName::parse(group) {
                    predefined_groups.push((group_name, index));
                }
            }
        }

        let mut used_custom_groups: Vec<(Vec<Regex>, usize)> =
            Vec::with_capacity(used_custom_group_index_map.len());
        for custom_group in custom_groups {
            if let Some(index) = used_custom_group_index_map.get(&custom_group.group_name) {
                let patterns = custom_group
                    .element_name_pattern
                    .iter()
                    .filter_map(|p| {
                        let anchored = if p.starts_with('^') {
                            p.to_owned()
                        } else {
                            format!("^{}", p)
                        };
                        match Regex::new(&anchored) {
                            Ok(regex) => Some(regex),
                            Err(_) => None,
                        }
                    })
                    .collect::<Vec<_>>();

                if !patterns.is_empty() {
                    used_custom_groups.push((patterns, *index));
                }
            }
        }

        predefined_groups.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            custom_groups: used_custom_groups,
            predefined_groups,
            unknown_group_index: unknown_group_index.unwrap_or(groups.len()),
        }
    }

    pub fn compute_group_index(&self, import_metadata: &ImportMetadata) -> usize {
        for (patterns, index) in &self.custom_groups {
            let is_match = patterns.iter().any(|regex| regex.is_match(import_metadata.source));
            if is_match {
                return *index;
            }
        }

        for (group_name, index) in &self.predefined_groups {
            if group_name
                .is_a_possible_name_of(&import_metadata.selectors, &import_metadata.modifiers)
            {
                return *index;
            }
        }

        self.unknown_group_index
    }

    pub fn should_regroup_side_effect(&self) -> bool {
        self.predefined_groups
            .iter()
            .any(|(group, _)| group.is_plain_selector(ImportSelector::SideEffect))
    }
    pub fn should_regroup_side_effect_style(&self) -> bool {
        self.predefined_groups
            .iter()
            .any(|(group, _)| group.is_plain_selector(ImportSelector::SideEffectStyle))
    }
}
