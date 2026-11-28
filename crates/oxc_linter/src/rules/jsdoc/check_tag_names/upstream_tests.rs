//! Conformance with eslint-plugin-jsdoc's checkTagNames test suite.
//! <https://github.com/gajus/eslint-plugin-jsdoc/blob/7e9acbf2143fd98b61ea406b7ea8d723b55002ef/test/rules/assertions/checkTagNames.js>

use std::{path::Path, sync::Arc};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::{
    ConfigStore, ConfigStoreBuilder, ContextSubHost, ContextSubHostOptions, ExternalPluginStore,
    FixKind, Fixer, LintOptions, Linter, ModuleRecord,
};

#[derive(Deserialize)]
struct Suite {
    valid: Vec<Case>,
    invalid: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    code: String,
    filename: Option<String>,
    #[serde(default)]
    options: Vec<Value>,
    #[serde(default)]
    settings: Map<String, Value>,
    #[serde(default)]
    errors: Vec<ExpectedError>,
    output: Option<String>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct ExpectedError {
    line: usize,
    message: String,
}

#[test]
fn check_tag_names_upstream() {
    let suite: Suite =
        serde_json::from_str(include_str!("../../../../fixtures/jsdoc/check-tag-names.json"))
            .unwrap();
    for (group, cases) in [("valid", suite.valid), ("invalid", suite.invalid)] {
        for (index, case) in cases.into_iter().enumerate() {
            let case_id = format!("{group} #{}", index + 1);
            check_case(&case, &case_id);
        }
    }
}

fn check_case(case: &Case, case_id: &str) {
    let mut rule = vec![json!("warn")];
    rule.extend(case.options.iter().cloned());
    let config = serde_json::from_value(json!({
        "plugins": ["jsdoc"],
        "categories": {},
        "rules": {"jsdoc/check-tag-names": rule},
        "settings": case.settings,
    }))
    .unwrap();
    let mut plugins = ExternalPluginStore::default();
    let store = ConfigStoreBuilder::from_oxlintrc(true, config, None, &mut plugins, None)
        .unwrap()
        .build(&mut plugins)
        .unwrap();
    let linter = Linter::new(
        LintOptions::default(),
        ConfigStore::new(store, FxHashMap::default(), plugins),
        None,
    )
    .with_fix(FixKind::SafeFix);

    let path = Path::new(case.filename.as_deref().unwrap_or("file.js"));
    let source_type = SourceType::from_path(path).unwrap().with_module(true);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &case.code, source_type).parse();
    // These two upstream fixtures exercise @abstract in ambient contexts. The
    // TypeScript ESLint parser accepts their input; Oxc diagnoses the invalid
    // statements but still returns the AST needed to exercise the rule.
    let mut parser_errors: Vec<_> = parsed.diagnostics.iter().map(|d| d.code.to_string()).collect();
    parser_errors.sort_unstable();
    let expected_parser_errors: &[&str] = match case_id {
        "valid #5" => &["TS(1036)"],
        "valid #6" => &["TS(1038)", "TS(1183)"],
        _ => &[],
    };
    assert!(!parsed.fatal_error, "{case_id}");
    assert_eq!(parser_errors, expected_parser_errors, "{case_id}");

    let built = SemanticBuilder::new_linter()
        .with_check_syntax_error(false)
        .build(allocator.alloc(parsed.program));
    assert!(built.diagnostics.is_empty(), "{case_id}: {:?}", built.diagnostics);
    let module = ModuleRecord::new(path, &parsed.module_record, &built.semantic);
    let context =
        ContextSubHost::new(built.semantic, Arc::new(module), 0, ContextSubHostOptions::default());
    let mut messages = linter.run(path, vec![context], &allocator);
    messages.sort_by_key(|message| message.span.start);
    let errors: Vec<_> = messages
        .iter()
        .map(|message| ExpectedError {
            line: 1 + case.code[..message.span.start as usize].matches('\n').count(),
            message: message.error.message.to_string(),
        })
        .collect();
    assert_eq!(errors, case.errors, "{case_id}");
    let fixed = Fixer::new(&case.code, messages, Some(source_type)).fix();
    assert_eq!(fixed.fixed_code, case.output.as_deref().unwrap_or(&case.code), "{case_id}");
}
