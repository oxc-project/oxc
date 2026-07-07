//! Shared functionality for the `compat` plugin: a native port of
//! [eslint-plugin-compat](https://github.com/amilajack/eslint-plugin-compat).

pub mod data;
pub mod rule_maps;
pub mod support;
pub mod targets;

pub use data::COMPAT_DATA;
pub use rule_maps::{FailingRule, RuleMap, RuleMaps, get_rules_for_targets};
pub use targets::{BrowserTarget, determine_targets_from_config, parse_browserslist_version};
