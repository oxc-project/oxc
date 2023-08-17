use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    rc::Rc,
    result::Result,
    sync::Arc,
};

use ignore::Walk;
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
    Report,
};
use oxc_linter::LintContext;
use oxc_parser::Parser;
use oxc_query::{schema, Adapter};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::{SourceType, Span};
use serde::Deserialize;
use trustfall::{execute_query, FieldValue, Schema, TransparentValue, TryIntoStruct};

enum SpanInfo {
    SingleSpanInfo(SingleSpanInfo),
    MultipleSpanInfo(MultipleSpanInfo),
}

#[derive(Debug, Deserialize)]
struct SingleSpanInfo {
    span_start: u64,
    span_end: u64,
}

#[derive(Debug, Deserialize)]
struct MultipleSpanInfo {
    span_start: Box<[u64]>,
    span_end: Box<[u64]>,
}

#[derive(Deserialize, Clone)]
pub struct InputQuery {
    pub name: String,
    pub query: String,
    pub args: BTreeMap<Arc<str>, TransparentValue>,
    pub summary: String,
    pub reason: String,
    #[serde(default)]
    pub tests: QueryTests,
}

#[derive(Deserialize, Default, Clone)]
pub struct QueryTests {
    pub pass: Vec<SingleTest>,
    pub fail: Vec<SingleTest>,
}

#[derive(Deserialize, Clone)]
pub struct SingleTest {
    pub relative_path: Vec<String>,
    pub code: String,
}

pub struct LinterPlugin {
    pub(super) rules: Vec<InputQuery>,
    schema: &'static Schema,
}

pub enum RulesToRun {
    All,
    Only(String),
}

#[derive(Debug, Error, Diagnostic)]
#[error("{0}")]
pub struct LinterPluginError(String, String, #[label("{1}")] Span);

impl LinterPlugin {
    pub fn new(schema: &'static Schema, queries_path: PathBuf) -> Self {
        let rules = Walk::new(queries_path)
            .filter_map(Result::ok)
            .filter(|f| {
                Path::new(f.path().as_os_str().to_str().unwrap())
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("yml"))
            })
            .map(|f| fs::read_to_string(f.path()))
            .map(Result::unwrap)
            .map(|rule| {
                serde_yaml::from_str::<InputQuery>(rule.as_str())
                    .unwrap_or_else(|_| panic!("{rule}\n\nQuery above"))
            })
            .collect::<Vec<_>>();

        Self { rules, schema }
    }

    #[allow(clippy::redundant_allocation)]
    pub fn run_plugin_rules(
        &self,
        ctx: &mut LintContext,
        plugin: &InputQuery,
        adapter: &Arc<&Adapter<'_>>,
    ) {
        for data_item in execute_query(
                self.schema,
                Arc::clone(adapter),
                &plugin.query,
                plugin.args.clone(),
            )
            .unwrap_or_else(|err| {
                panic!(
                    "not a legal query in query: \n\n\n{}\n\n\nErr: {err}",
                    plugin.query.as_str()
                )
            })
            .map(|v| {
                if env::var("OXC_PRINT_TRUSTFALL_OUTPUTS").unwrap_or_else(|_| "false".to_owned())
                    == "true"
                {
                    println!("{v:#?}");
                }
                match (v.get("span_start"), v.get("span_end")) {
                    (Some(FieldValue::List(x)), Some(FieldValue::List(y))) if matches!(x[0], FieldValue::Int64(_)) && matches!(y[0], FieldValue::Int64(_)) => {
                        v.try_into_struct::<MultipleSpanInfo>().map(SpanInfo::MultipleSpanInfo).expect("to be able to convert into MultipleSpanInfo")
                    }
                    (Some(FieldValue::List(x)), Some(FieldValue::List(y))) if matches!(x[0], FieldValue::Uint64(_)) && matches!(y[0], FieldValue::Uint64(_)) => {
                        v.try_into_struct::<MultipleSpanInfo>().map(SpanInfo::MultipleSpanInfo).expect("to be able to convert into MultipleSpanInfo")
                    }
                    (Some(FieldValue::Int64(_)), Some(FieldValue::Int64(_))) | (Some(FieldValue::Uint64(_)), Some(FieldValue::Uint64(_))) => {
                        v.try_into_struct::<SingleSpanInfo>().map(SpanInfo::SingleSpanInfo).expect("to be able to convert into SingleSpanInfo")
                    },
                    (None, None) => panic!("No `span_start` and `span_end` were not `@output`'d from query '{}'", plugin.name),
                    (a, b) => panic!("Wrong type for `span_start` and `span_end` in query '{}'. Expected both to be Int or list of Int.\nInstead got:\nspan_start={a:?} & span_end={b:?}", plugin.name),
                }
            })
            .take(usize::MAX)
            {
                ctx.with_rule_name("a rule");
                // TODO: this isn't how we do this at all, need to make this consistent with the project's miette style
                match data_item {
                    SpanInfo::SingleSpanInfo(SingleSpanInfo {
                        span_start: start,
                        span_end: end,
                    }) => {
                        ctx.diagnostic(LinterPluginError(plugin.summary.clone(), plugin.reason.clone(), Span{ start: start.try_into().unwrap(), end: end.try_into().unwrap() }));
                    }
                    SpanInfo::MultipleSpanInfo(MultipleSpanInfo {
                        span_start: start,
                        span_end: end,
                    }) => {
                        for i in 0..start.len() {
                            ctx.diagnostic(LinterPluginError(plugin.summary.clone(), plugin.reason.clone(), Span{ start: start[i].try_into().unwrap(), end: end[i].try_into().unwrap() }));
                        }
                    }
                }
            }
    }

    pub fn run_tests(
        &self,
        ctx: &mut LintContext,
        relative_file_path_parts: Vec<Option<String>>,
        rules_to_run: RulesToRun,
    ) {
        let inner = Adapter::new(Rc::clone(ctx.semantic()), relative_file_path_parts);
        let adapter = Arc::from(&inner);
        if let RulesToRun::Only(this_rule) = rules_to_run {
            self.rules
                .iter()
                .filter(|x| x.name == this_rule)
                .for_each(|rule| self.run_plugin_rules(ctx, rule, &adapter));
        } else {
            self.rules.iter().for_each(|rule| self.run_plugin_rules(ctx, rule, &adapter));
        }
    }
}

fn run_test(test: &SingleTest, rule_name: &str, plugin: &LinterPlugin) -> Vec<Report> {
    let file_path = &test.relative_path.last().expect("there to be atleast 1 path part");
    let source_text = &test.code;

    let allocator = Allocator::default();
    let source_type = SourceType::from_path(file_path).unwrap();
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    // Handle parser errors
    assert!(
        ret.errors.is_empty(),
        "In test {rule_name}: Parser errors: {:?} Code:\n\n{:?}",
        ret.errors,
        test.code
    );

    let program = allocator.alloc(ret.program);
    let SemanticBuilderReturn { semantic, errors } =
        SemanticBuilder::new(source_text, source_type).with_trivias(ret.trivias).build(program);

    assert!(
        errors.is_empty(),
        "In test {rule_name}: Semantic errors: {:?} Code:\n\n{:?}",
        errors,
        test.code
    );

    let semantic = Rc::new(semantic);

    let mut lint_ctx = LintContext::new(&Rc::clone(&semantic));

    plugin.run_tests(
        &mut lint_ctx,
        test.relative_path.iter().map(|el| Some(el.clone())).collect::<Vec<_>>(),
        RulesToRun::Only(rule_name.to_string()),
    );

    lint_ctx.into_message().into_iter().map(|m| m.error).collect::<Vec<_>>()
}

pub fn test_queries(queries_to_test: PathBuf) {
    let plugin = LinterPlugin::new(schema(), queries_to_test);

    for rule in &plugin.rules {
        for (i, test) in rule.tests.pass.iter().enumerate() {
            let messages = run_test(test, &rule.name, &plugin);
            assert!(
                    messages.is_empty(),
                    "{}'s test {} is failing when it should pass.\nErrors: {:#?}\nPath: {:?}\nCode:\n\n{}\n\n",
                    rule.name,
                    i + 1,
                    messages,
                    test.relative_path,
                    test.code
                );
        }

        for (i, test) in rule.tests.fail.iter().enumerate() {
            let messages = run_test(test, &rule.name, &plugin);
            assert!(
                !messages.is_empty(),
                "{}'s test {} is passing when it should fail.\nPath: {:?}\nCode:\n\n{}\n\n",
                rule.name,
                i + 1,
                test.relative_path,
                test.code
            );
        }

        if rule.tests.pass.len() + rule.tests.fail.len() > 0 {
            println!(
                "{} passed {} tests successfully.\n",
                rule.name,
                rule.tests.pass.len() + rule.tests.fail.len()
            );
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::test_queries;

    #[test]
    fn query_tests() {
        test_queries(Path::new("examples/queries").to_path_buf());
    }
}
