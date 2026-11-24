use std::{collections::BTreeSet, fs, path::Path};

use cow_utils::CowUtils;

use oxc::{
    allocator::Allocator,
    ast::ast::{BindingPattern, Expression, ObjectPropertyKind, PropertyKey, Statement},
    parser::Parser,
    span::{GetSpan, SourceType},
};

use crate::{
    TestRunnerOptions, conformance_root, expanded_test_root, oxc_test_root, test_case::TestCase,
};

const PLUGIN: &str = "babel-plugin-proposal-decorators";
const CASE_COUNT: usize = 147;

pub fn run(options: &TestRunnerOptions) {
    let plugin = oxc_test_root().join(PLUGIN);
    let source = fs::read_to_string(plugin.join("upstream/decorator-tests.ts")).unwrap();
    let suite = Suite::parse(&source);
    assert_eq!(suite.cases.len(), CASE_COUNT, "esbuild decorator case count changed");

    let root = expanded_test_root();
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    for group in ["esbuild", "esbuild-native"] {
        let relative = Path::new(PLUGIN).join("test/fixtures").join(group);
        let directory = root.join(&relative);
        fs::create_dir_all(&directory).unwrap();
        fs::copy(
            plugin.join("test/fixtures").join(group).join("options.json"),
            directory.join("options.json"),
        )
        .unwrap();
        for case in &suite.cases {
            let relative = relative.join(&case.slug).join("exec.ts");
            if options.filter.as_ref().is_some_and(|filter| {
                !oxc_test_root().join(&relative).to_string_lossy().contains(filter)
            }) {
                continue;
            }
            let path = root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, suite.code(case)).unwrap();
            TestCase::new(&conformance_root(), &path).unwrap().test(options);
        }
    }
}

struct Suite<'s> {
    preamble: &'s str,
    helpers: &'s str,
    cases: Vec<Case<'s>>,
}

struct Case<'s> {
    slug: String,
    key: &'s str,
    callback: &'s str,
}

impl<'s> Suite<'s> {
    fn parse(source: &'s str) -> Self {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let declaration = parsed.program.body.iter().find_map(|statement| {
            if let Statement::VariableDeclaration(declaration) = statement
                && declaration.declarations.iter().any(|declarator| {
                    matches!(&declarator.id, BindingPattern::BindingIdentifier(id) if id.name == "tests")
                })
            {
                Some(declaration)
            } else {
                None
            }
        }).expect("esbuild suite must declare its tests object");
        assert_eq!(declaration.declarations.len(), 1);
        let Some(Expression::ObjectExpression(object)) = &declaration.declarations[0].init else {
            panic!("esbuild tests must be an object literal");
        };
        let runner = parsed
            .program
            .body
            .iter()
            .find_map(|statement| {
                if let Statement::FunctionDeclaration(function) = statement
                    && function.id.as_ref().is_some_and(|id| id.name == "run")
                {
                    Some(function)
                } else {
                    None
                }
            })
            .expect("esbuild suite must declare its run function");
        let preamble = &source[..declaration.span.start as usize];
        let helpers = &source[declaration.span.end as usize..runner.span.start as usize];
        let mut slugs = BTreeSet::new();
        let cases = object
            .properties
            .iter()
            .map(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    panic!("esbuild tests cannot contain spread properties");
                };
                let PropertyKey::StaticStringLiteral(key) = &property.key else {
                    panic!("esbuild test names must be string literals");
                };
                assert!(!property.method);
                assert!(
                    matches!(&property.value, Expression::ArrowFunctionExpression(_)),
                    "esbuild test must be an arrow callback"
                );
                let name = key.value.as_str().expect("esbuild test names must be valid UTF-8");
                let slug = name
                    .split(|c: char| !c.is_ascii_alphanumeric())
                    .filter(|part| !part.is_empty())
                    .map(CowUtils::cow_to_ascii_lowercase)
                    .collect::<Vec<_>>()
                    .join("-");
                assert!(
                    !slug.is_empty() && slugs.insert(slug.clone()),
                    "duplicate esbuild test name: {name}"
                );
                Case {
                    slug,
                    key: property.key.span().source_text(source),
                    callback: property.value.span().source_text(source),
                }
            })
            .collect();
        Self { preamble, helpers, cases }
    }

    fn code(&self, case: &Case<'_>) -> String {
        // Keep the upstream assertion helpers, including their strict equality check.
        // A failed check increments `failures`; make that a Vitest module failure.
        format!(
            "{}{}\ntestName = {};\nconst run = {};\nawait run();\nexpect(failures).toBe(0);\nexport {{}};\n",
            self.preamble, self.helpers, case.key, case.callback
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str =
        include_str!("../tests/babel-plugin-proposal-decorators/upstream/decorator-tests.ts");

    #[test]
    fn includes_every_upstream_callback_and_awaits_async_cases() {
        let suite = Suite::parse(SOURCE);
        assert_eq!(suite.cases.len(), CASE_COUNT);
        assert_eq!(
            suite.cases.iter().filter(|case| case.callback.starts_with("async ")).count(),
            2
        );
        assert!(suite.preamble.contains("Symbol.metadata"));
        assert!(suite.helpers.contains("if (x === expected) return true"));
        assert!(suite.helpers.contains("if (error instanceof expected) return true"));
        assert!(suite.helpers.contains("let failures = 0"));
        assert!(!suite.helpers.contains("async function run()"));
        let allocator = Allocator::default();
        for case in &suite.cases {
            let code = suite.code(case);
            assert!(code.contains(case.callback));
            assert!(code.contains("await run();\nexpect(failures).toBe(0);"));
            let parsed = Parser::new(&allocator, &code, SourceType::ts()).parse();
            assert!(parsed.diagnostics.is_empty(), "{}: {:?}", case.slug, parsed.diagnostics);
        }
    }

    #[test]
    #[should_panic(expected = "duplicate esbuild test name")]
    fn rejects_names_that_would_overwrite_a_generated_test() {
        Suite::parse(
            "const tests = { 'Same name': () => {}, 'Same-name': async () => {} }; async function run() {}",
        );
    }

    #[test]
    #[should_panic(expected = "esbuild tests cannot contain spread properties")]
    fn rejects_properties_that_cannot_be_expanded() {
        Suite::parse("const tests = { ...otherTests }; async function run() {}");
    }
}
