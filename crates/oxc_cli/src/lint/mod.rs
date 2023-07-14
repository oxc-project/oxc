mod command;
mod error;
mod isolated_handler;
mod module_tree_handler;
mod options;

use std::{io::BufWriter, sync::Arc};

use oxc_index::assert_impl_all;
use oxc_linter::{Linter, RuleCategory, RuleEnum, RULES};
use rustc_hash::FxHashSet;

pub use self::{error::Error, options::LintOptions};
use self::{
    isolated_handler::IsolatedLintHandler, module_tree_handler::ModuleTreeLintHandler,
    options::AllowWarnDeny,
};
use crate::{CliRunResult, Runner};

pub struct LintRunner {
    options: Arc<LintOptions>,
    linter: Arc<Linter>,
}
assert_impl_all!(LintRunner: Send, Sync);

impl Default for LintRunner {
    fn default() -> Self {
        Self::new(LintOptions::default())
    }
}

impl Runner for LintRunner {
    type Options = LintOptions;

    const ABOUT: &'static str = "Lint this repository.";
    const NAME: &'static str = "lint";

    fn new(options: LintOptions) -> Self {
        let linter = Linter::from_rules(Self::derive_rules(&options)).with_fix(options.fix);
        Self { options: Arc::new(options), linter: Arc::new(linter) }
    }

    fn run(&self) -> CliRunResult {
        if self.options.list_rules {
            Self::print_rules();
            return CliRunResult::None;
        }

        if Self::enable_module_tree() {
            ModuleTreeLintHandler::new(Arc::clone(&self.options), Arc::clone(&self.linter)).run()
        } else {
            IsolatedLintHandler::new(Arc::clone(&self.options), Arc::clone(&self.linter)).run()
        }
    }
}

impl LintRunner {
    /// check if the module tree should be provided when linting
    fn enable_module_tree() -> bool {
        matches!(std::env::var("OXC_MODULE_TREE"), Ok(x) if x == "true" || x == "1")
    }

    pub fn print_rules() {
        let mut stdout = BufWriter::new(std::io::stdout());
        Linter::print_rules(&mut stdout);
    }

    fn derive_rules(options: &LintOptions) -> Vec<RuleEnum> {
        let mut rules: FxHashSet<RuleEnum> = FxHashSet::default();

        for (allow_warn_deny, name_or_category) in &options.rules {
            let maybe_category = RuleCategory::from(name_or_category.as_str());
            match allow_warn_deny {
                AllowWarnDeny::Deny => {
                    match maybe_category {
                        Some(category) => rules.extend(
                            RULES.iter().filter(|rule| rule.category() == category).cloned(),
                        ),
                        None => {
                            if name_or_category == "all" {
                                rules.extend(RULES.iter().cloned());
                            } else {
                                rules.extend(
                                    RULES
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
                                rules.retain(|rule| rule.name() == name_or_category);
                            }
                        }
                    };
                }
            }
        }

        let mut rules = rules.into_iter().collect::<Vec<_>>();
        // for stable diagnostics output ordering
        rules.sort_unstable_by_key(|rule| rule.name());
        rules
    }
}
