use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    ir_transform::sort_imports::group_config::{GroupName, ImportModifier, ImportSelector},
    oxfmtrc::CustomGroupDefinition,
};

// intermediate import metadata that is used for group matching
pub struct ImportMetadata<'a> {
    pub source: &'a str,
    pub selectors: Vec<ImportSelector>,
    pub modifiers: Vec<ImportModifier>,
}

pub struct GroupMatcher {
    // custom groups that are used in options.groups.
    pub custom_groups: Vec<(CustomGroupDefinition, usize)>,

    // > Predefined groups are characterized by a single selector and potentially multiple modifiers.
    // > You may enter modifiers in any order, but the selector must always come at the end.

    // predefined groups sorted by priority
    // so that we don't need to enumerate all possible group names of a given import.
    pub predefined_groups: Vec<(GroupName, usize)>,

    // The index of "unknown" in groups or groups.len() if absent
    pub unknown_group_index: usize,
}

impl GroupMatcher {
    pub fn new(groups: &Vec<Vec<String>>, custom_groups: &Vec<CustomGroupDefinition>) -> Self {
        let custom_group_name_set =
            FxHashSet::from_iter(custom_groups.iter().map(|g| g.name.clone()));

        let mut unknown_group_index: Option<usize> = None;

        let mut used_custom_group_index_map = FxHashMap::default();
        let mut predefined_groups = Vec::new();
        for (index, group_union) in groups.iter().enumerate() {
            for group in group_union.iter() {
                if group == "unknown" {
                    unknown_group_index = Some(index);
                } else if custom_group_name_set.contains(group) {
                    used_custom_group_index_map.insert(group.to_owned(), index);
                } else if let Some(group_name) = GroupName::parse(group) {
                    predefined_groups.push((group_name, index));
                }
            }
        }

        let mut used_custom_groups: Vec<(CustomGroupDefinition, usize)> =
            Vec::with_capacity(used_custom_group_index_map.len());
        for custom_group in custom_groups.iter() {
            if let Some(index) = used_custom_group_index_map.get(&custom_group.name) {
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

    pub fn compute_group_index(&self, import_metadata: &ImportMetadata) -> usize {
        for (custom_group, index) in self.custom_groups.iter() {
            if custom_group.does_match(import_metadata) {
                return *index;
            }
        }

        for (group_name, index) in self.predefined_groups.iter() {
            if group_name
                .is_a_possible_name_of(&import_metadata.selectors, &import_metadata.modifiers)
            {
                return *index;
            }
        }

        self.unknown_group_index
    }

    // ref: https://github.com/oxc-project/oxc/blob/92003083000b854658dee57462f3f12588b2d1df/crates/oxc_formatter/src/ir_transform/sort_imports/compute_metadata.rs#L56-L67
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

impl CustomGroupDefinition {
    pub fn does_match(&self, import_metadata: &ImportMetadata) -> bool {
        for rule in self.any_of.iter() {
            if rule.selector.as_ref().is_some_and(|s| {
                ImportSelector::parse(&s)
                    .is_some_and(|selector| !import_metadata.selectors.contains(&selector))
            }) {
                continue;
            }
            if rule.modifiers.as_ref().is_some_and(|modifiers| {
                !modifiers.iter().all(|m| {
                    ImportModifier::parse(m)
                        .is_some_and(|modifier| import_metadata.modifiers.contains(&modifier))
                })
            }) {
                continue;
            }
            if rule
                .element_name_pattern
                .as_ref()
                .is_some_and(|pattern| !import_metadata.source.starts_with(pattern))
            {
                continue;
            }
            return true;
        }
        false
    }
}
