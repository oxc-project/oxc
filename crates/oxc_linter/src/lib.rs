#![feature(let_chains, is_some_and)]

#[cfg(test)]
mod tester;

mod autofix;
mod context;
pub mod rule;
mod rules;

use std::{borrow::Cow, fs, rc::Rc};

use oxc_diagnostics::Error;
pub(crate) use oxc_semantic::AstNode;
use oxc_semantic::Semantic;

use crate::{
    autofix::Fixer,
    context::LintContext,
    rules::{RuleEnum, RULES},
};

#[derive(Debug)]
pub struct LintRunResult<'a> {
    pub fixed_source: Cow<'a, str>,
    pub diagnostics: Vec<Error>,
}

impl<'a> From<LintContext<'a>> for LintRunResult<'a> {
    fn from(ctx: LintContext<'a>) -> Self {
        let source_text = ctx.source_text();
        let (fixes, diagnostics) = ctx.into_diagnostics();
        Self { fixed_source: Fixer::new(source_text, fixes).fix(), diagnostics }
    }
}

#[derive(Debug)]
pub struct Linter {
    rules: Vec<RuleEnum>,
}

impl Linter {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let rules_config = Self::read_rules_configuration();
        let rules = rules_config.map_or_else(
            || RULES.to_vec(),
            |rules_config| {
                RULES
                    .iter()
                    .map(|rule| {
                        let value = rules_config.get(rule.name());
                        rule.read_json(value.cloned())
                    })
                    .collect()
            },
        );
        Self { rules }
    }

    #[must_use]
    pub fn from_rules(rules: Vec<RuleEnum>) -> Self {
        Self { rules }
    }

    #[must_use]
    pub fn run<'a>(
        &self,
        semantic: &Rc<Semantic<'a>>,
        source_text: &'a str,
        fix: bool,
    ) -> LintRunResult<'a> {
        let ctx = LintContext::new(source_text, semantic.clone(), fix);

        for node in semantic.nodes().iter() {
            for rule in &self.rules {
                rule.run(node, &ctx);
            }
        }

        ctx.into()
    }

    fn read_rules_configuration() -> Option<serde_json::Map<String, serde_json::Value>> {
        fs::read_to_string(".eslintrc.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .and_then(|v: serde_json::Value| v.get("rules").cloned())
            .and_then(|v| v.as_object().cloned())
    }
}
