#![allow(clippy::self_named_module_files)] // for rules.rs
#![feature(let_chains, const_trait_impl, const_slice_index)]

#[cfg(test)]
mod tester;

mod ast_util;
mod context;
mod disable_directives;
mod fixer;
mod globals;
mod lint_adapter;
pub mod rule;
mod rules;

use std::{collections::BTreeMap, fs, io::Write, rc::Rc, sync::Arc};

pub use fixer::{FixResult, Fixer, Message};
use lazy_static::lazy_static;
use lint_adapter::{InputQuery, LintAdapter};
use oxc_diagnostics::miette::{miette, LabeledSpan};
pub(crate) use oxc_semantic::AstNode;
use oxc_semantic::Semantic;
use rustc_hash::FxHashMap;
use trustfall::{execute_query, Schema, TransparentValue};

use crate::context::LintContext;
pub use crate::{
    rule::RuleCategory,
    rules::{RuleEnum, RULES},
};

lazy_static! {
    static ref TRUSTFALL_RULES: Vec<InputQuery> = fs::read_dir("queries")
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|dir_entry| dir_entry.path().is_file())
        .filter(|f| f.path().as_os_str().to_str().unwrap().ends_with(".ron"))
        .map(|f| fs::read_to_string(f.path()))
        .map(std::result::Result::unwrap)
        .map(|rule| { ron::from_str::<InputQuery>(rule.as_str()).unwrap() })
        .collect();
    static ref SCHEMA: Schema = match Schema::parse(fs::read_to_string("./schema.graphql").unwrap())
    {
        Ok(schema) => schema,
        Err(err) => {
            println!("{}", err);
            unimplemented!()
        }
    };
}

#[derive(Debug)]
pub struct Linter {
    rules: Vec<RuleEnum>,

    fix: bool,
}

impl Linter {
    pub fn new() -> Self {
        let rules = RULES
            .iter()
            .cloned()
            .filter(|rule| rule.category() == RuleCategory::Correctness)
            .collect::<Vec<_>>();
        Self::from_rules(rules)
    }

    pub fn from_rules(rules: Vec<RuleEnum>) -> Self {
        Self { rules, fix: false }
    }

    pub fn has_fix(&self) -> bool {
        self.fix
    }

    pub fn number_of_rules(&self) -> usize {
        self.rules.len()
    }

    #[must_use]
    pub fn with_fix(mut self, yes: bool) -> Self {
        self.fix = yes;
        self
    }

    pub fn from_json_str(s: &str) -> Self {
        let rules = serde_json::from_str(s)
            .ok()
            .and_then(|v: serde_json::Value| v.get("rules").cloned())
            .and_then(|v| v.as_object().cloned())
            .map_or_else(
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

        Self::from_rules(rules)
    }

    pub fn run<'a>(&self, semantic: &Rc<Semantic<'a>>) -> Vec<Message<'a>> {
        let mut ctx = LintContext::new(semantic, self.fix);
        for node in semantic.nodes().iter() {
            for rule in &self.rules {
                ctx.with_rule_name(rule.name());
                rule.run(node, &ctx);
            }
        }

        let adapter = LintAdapter { semantic: semantic.clone() };
        let la = Arc::from(adapter);
        for input_query in TRUSTFALL_RULES.iter() {
            for data_item in execute_query(
                &SCHEMA,
                Arc::clone(&la),
                input_query.query.as_str(),
                input_query.args.clone(),
            )
            .expect(
                format!("not a legal query in query: \n\n\n{}", input_query.query.as_str())
                    .as_str(),
            )
            .take(usize::MAX)
            {
                let transparent: BTreeMap<_, TransparentValue> =
                    data_item.into_iter().map(|(k, v)| (k, v.into())).collect();
                // println!("\n{}", serde_json::to_string_pretty(&transparent).unwrap());
                ctx.with_rule_name("a rule");
                let TransparentValue::Uint64(start) = &transparent["span_start"] else {
                        println!("{:?}", transparent);
                        unreachable!()
                    };
                let TransparentValue::Uint64(end) = &transparent["span_end"] else {
                        println!("{:?}", transparent);
                        unreachable!()
                    };
                let c = miette!(
                    labels = vec![LabeledSpan::at(
                        (*start as usize, (*end - *start) as usize),
                        input_query.reason.as_str()
                    )],
                    "Unexpected error"
                );
                ctx.diagnostic(c);
            }
        }

        for symbol in semantic.symbols().iter() {
            for rule in &self.rules {
                rule.run_on_symbol(symbol, &ctx);
            }
        }

        ctx.into_message()
    }

    #[allow(unused)]
    fn read_rules_configuration() -> Option<serde_json::Map<String, serde_json::Value>> {
        fs::read_to_string(".eslintrc.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .and_then(|v: serde_json::Value| v.get("rules").cloned())
            .and_then(|v| v.as_object().cloned())
    }

    pub fn print_rules<W: Write>(writer: &mut W) {
        let rules_by_category = RULES.iter().fold(FxHashMap::default(), |mut map, rule| {
            map.entry(rule.category()).or_insert_with(Vec::new).push(rule);
            map
        });

        for (category, rules) in rules_by_category {
            writeln!(writer, "{} ({}):", category, rules.len()).unwrap();
            for rule in rules {
                writeln!(writer, "• {}/{}", rule.plugin_name(), rule.name()).unwrap();
            }
        }
        writeln!(writer, "Total: {}", RULES.len()).unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::Linter;

    #[test]
    fn print_rules() {
        let mut writer = Vec::new();
        Linter::print_rules(&mut writer);
        assert!(!writer.is_empty());
    }
}
