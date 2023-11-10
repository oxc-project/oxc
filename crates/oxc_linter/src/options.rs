use crate::{RuleCategory, RuleEnum, RULES};
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct LintOptions {
    /// Allow / Deny rules in order. [("allow" / "deny", rule name)]
    /// Defaults to [("deny", "correctness")]
    pub filter: Vec<(AllowWarnDeny, String)>,
    pub fix: bool,
    pub timing: bool,
    pub import_plugin: bool,
    pub jest_plugin: bool,
    pub jsx_a11y_plugin: bool,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            filter: vec![(AllowWarnDeny::Deny, String::from("correctness"))],
            fix: false,
            timing: false,
            import_plugin: false,
            jest_plugin: false,
            jsx_a11y_plugin: false,
        }
    }
}

impl LintOptions {
    #[must_use]
    pub fn with_filter(mut self, filter: Vec<(AllowWarnDeny, String)>) -> Self {
        if !filter.is_empty() {
            self.filter = filter;
        }
        self
    }

    #[must_use]
    pub fn with_fix(mut self, yes: bool) -> Self {
        self.fix = yes;
        self
    }

    #[must_use]
    pub fn with_timing(mut self, yes: bool) -> Self {
        self.timing = yes;
        self
    }

    #[must_use]
    pub fn with_import_plugin(mut self, yes: bool) -> Self {
        self.import_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_jest_plugin(mut self, yes: bool) -> Self {
        self.jest_plugin = yes;
        self
    }

    #[must_use]
    pub fn with_jsx_a11y_plugin(mut self, yes: bool) -> Self {
        self.jsx_a11y_plugin = yes;
        self
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AllowWarnDeny {
    Allow,
    // Warn,
    Deny,
}

impl From<&'static str> for AllowWarnDeny {
    fn from(s: &'static str) -> Self {
        match s {
            "allow" => Self::Allow,
            "deny" => Self::Deny,
            _ => unreachable!(),
        }
    }
}

const JEST_PLUGIN_NAME: &str = "jest";
const JSX_A11Y_PLUGIN_NAME: &str = "jsx_a11y";

impl LintOptions {
    pub fn derive_rules(&self) -> Vec<RuleEnum> {
        let mut rules: FxHashSet<RuleEnum> = FxHashSet::default();
        let all_rules = self.get_filtered_rules();

        for (allow_warn_deny, name_or_category) in &self.filter {
            let maybe_category = RuleCategory::from(name_or_category.as_str());
            match allow_warn_deny {
                AllowWarnDeny::Deny => {
                    match maybe_category {
                        Some(category) => rules.extend(
                            all_rules.iter().filter(|rule| rule.category() == category).cloned(),
                        ),
                        None => {
                            if name_or_category == "all" {
                                rules.extend(all_rules.iter().cloned());
                            } else {
                                rules.extend(
                                    all_rules
                                        .iter()
                                        .filter(|rule| rule.name() == name_or_category)
                                        .cloned(),
                                );
                            }
                        }
                    };
                }
                AllowWarnDeny::Allow => {
                    match maybe_category {
                        Some(category) => rules.retain(|rule| rule.category() != category),
                        None => {
                            if name_or_category == "all" {
                                rules.clear();
                            } else {
                                rules.retain(|rule| rule.name() != name_or_category);
                            }
                        }
                    };
                }
            }
        }

        let mut rules = rules.into_iter().collect::<Vec<_>>();
        // for stable diagnostics output ordering
        rules.sort_unstable_by_key(RuleEnum::name);
        rules
    }

    // get final filtered rules by reading `self.jest_plugin` and `self.jsx_a11y_plugin`
    fn get_filtered_rules(&self) -> Vec<RuleEnum> {
        let mut rules = RULES.clone();

        let mut may_exclude_plugin_rules = |yes: bool, name: &str| {
            if !yes {
                rules.retain(|rule| rule.plugin_name() != name);
            }
        };

        may_exclude_plugin_rules(self.jest_plugin, JEST_PLUGIN_NAME);
        may_exclude_plugin_rules(self.jsx_a11y_plugin, JSX_A11Y_PLUGIN_NAME);

        rules
    }
}
