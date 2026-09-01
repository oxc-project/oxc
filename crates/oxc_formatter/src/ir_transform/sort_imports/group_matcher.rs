use rustc_hash::FxHashMap;

use super::group_config::{GroupEntry, GroupName, ImportModifier, ImportSelector};
use super::options::CustomGroupDefinition;

// Intermediate import metadata that is used for group matching
pub struct ImportMetadata<'a> {
    pub source: &'a str,
    pub selectors: Vec<ImportSelector>,
    pub modifiers: Vec<ImportModifier>,
}

pub struct GroupMatcher {
    // Custom groups that are used in `options.groups`
    custom_groups: Vec<(CustomGroupDefinition, usize)>,
    // Predefined groups sorted by priority,
    // so that we don't need to enumerate all possible group names of a given import.
    predefined_groups: Vec<(GroupName, usize)>,
    // The index of "unknown" in groups or `groups.len()` if absent
    unknown_group_index: usize,
    // Whether a catch-all `side_effect(_style)` group is in use:
    // the explicit opt-in for regrouping side-effect imports even with `sort_side_effects: false`.
    // Either the predefined group, or a used custom group with the bare selector;
    // a custom group that merely happens to match (by pattern or modifiers) is NOT an opt-in.
    // Config-only facts, computed once here and queried per import.
    regroup_side_effect: bool,
    regroup_side_effect_style: bool,
}

impl GroupMatcher {
    pub fn new(groups: &[Vec<GroupEntry>], custom_groups: &[CustomGroupDefinition]) -> Self {
        let mut unknown_group_index: Option<usize> = None;

        let mut used_custom_group_index_map = FxHashMap::default();
        let mut predefined_groups = Vec::new();
        for (index, group_union) in groups.iter().enumerate() {
            for entry in group_union {
                match entry {
                    GroupEntry::Unknown => {
                        unknown_group_index = Some(index);
                    }
                    GroupEntry::Custom(name) => {
                        used_custom_group_index_map.insert(name.as_str(), index);
                    }
                    GroupEntry::Predefined(group_name) => {
                        predefined_groups.push((group_name.clone(), index));
                    }
                }
            }
        }

        let mut used_custom_groups: Vec<(CustomGroupDefinition, usize)> =
            Vec::with_capacity(used_custom_group_index_map.len());
        for custom_group in custom_groups {
            if let Some(index) = used_custom_group_index_map.get(custom_group.group_name.as_str()) {
                used_custom_groups.push((custom_group.clone(), *index));
            }
        }

        predefined_groups.sort_by(|a, b| a.0.cmp(&b.0));

        let has_catch_all_group_for = |selector: ImportSelector| {
            predefined_groups.iter().any(|(group, _)| group.is_plain_selector(selector))
                || used_custom_groups.iter().any(|(group, _)| group.is_plain_selector(selector))
        };
        let regroup_side_effect = has_catch_all_group_for(ImportSelector::SideEffect);
        let regroup_side_effect_style = has_catch_all_group_for(ImportSelector::SideEffectStyle);

        Self {
            custom_groups: used_custom_groups,
            predefined_groups,
            unknown_group_index: unknown_group_index.unwrap_or(groups.len()),
            regroup_side_effect,
            regroup_side_effect_style,
        }
    }

    pub fn compute_group_index(&self, import_metadata: &ImportMetadata) -> usize {
        for (custom_group, index) in &self.custom_groups {
            let is_match = {
                let name_matches = custom_group.element_name_pattern.is_empty()
                    || custom_group
                        .element_name_pattern
                        .iter()
                        .any(|pattern| fast_glob::glob_match(pattern, import_metadata.source));
                let selector_matches =
                    custom_group.selector.is_none_or(|s| import_metadata.selectors.contains(&s));
                let modifiers_match =
                    custom_group.modifiers.iter().all(|m| import_metadata.modifiers.contains(m));

                // These are AND logic
                name_matches && selector_matches && modifiers_match
            };

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
        self.regroup_side_effect
    }
    pub fn should_regroup_side_effect_style(&self) -> bool {
        self.regroup_side_effect_style
    }
}
