use std::{collections::BTreeMap, env, fs, rc::Rc, sync::Arc};

use oxc_diagnostics::miette::{miette, LabeledSpan};
use oxc_linter::LintContext;
use oxc_query::Adapter;
use oxc_semantic::Semantic;
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

impl LinterPlugin {
    pub fn new(schema: &'static Schema, queries_path: &str) -> Self {
        let rules = fs::read_dir(queries_path)
            .expect("to readdir queries_path folder")
            .filter_map(std::result::Result::ok)
            .filter(|dir_entry| dir_entry.path().is_dir())
            .filter(|dir| !dir.path().to_str().map_or_else(|| true, |f| f.contains("ignore_")))
            .filter_map(|folder| fs::read_dir(folder.path()).ok())
            .flat_map(std::iter::IntoIterator::into_iter)
            .filter_map(std::result::Result::ok)
            .filter(|dir_entry| dir_entry.path().is_file())
            .filter(|f| {
                std::path::Path::new(f.path().as_os_str().to_str().unwrap())
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("yml"))
            })
            .map(|f| fs::read_to_string(f.path()))
            .map(std::result::Result::unwrap)
            .map(|rule| {
                serde_yaml::from_str::<InputQuery>(rule.as_str())
                    .unwrap_or_else(|_| panic!("{rule}\n\nQuery above"))
            })
            .collect::<Vec<_>>();

        Self { rules, schema }
    }

    pub fn run(
        &self,
        ctx: &mut LintContext,
        semantic: Rc<Semantic<'_>>,
        relative_file_path_parts: Vec<Option<String>>,
        #[cfg(test)] only_run_rule_with_name: &str,
    ) {
        let inner = Adapter::new(semantic, relative_file_path_parts);
        let adapter = Arc::from(&inner);
        for input_query in &self.rules {
            #[cfg(test)]
            if input_query.name != only_run_rule_with_name {
                continue;
            }

            for data_item in execute_query(
                self.schema,
                Arc::clone(&adapter),
                &input_query.query,
                input_query.args.clone(),
            )
            .unwrap_or_else(|err| {
                panic!(
                    "not a legal query in query: \n\n\n{}\n\n\nErr: {err}",
                    input_query.query.as_str()
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
                    (Some(FieldValue::Int64(_)), Some(FieldValue::Int64(_))) => {
                        v.try_into_struct::<SingleSpanInfo>().map(SpanInfo::SingleSpanInfo).expect("to be able to convert into SingleSpanInfo")
                    }
                    (None, None) => panic!("No `span_start` and `span_end` were not `@output`'d from query '{}'", input_query.name),
                    _ => panic!("Wrong type for `span_start` and `span_end` in query '{}'. Expected both to be Int or list of Int.", input_query.name),
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
                        ctx.diagnostic(miette!(
                            labels = vec![LabeledSpan::at(
                                (
                                    usize::try_from(start)
                                        .expect("for start of span to fit in usize"),
                                    usize::try_from(end - start)
                                        .expect("for length of span to fit in usize")
                                ),
                                input_query.reason.as_str()
                            )],
                            "Unexpected error"
                        ));
                    }
                    SpanInfo::MultipleSpanInfo(MultipleSpanInfo {
                        span_start: start,
                        span_end: end,
                    }) => {
                        for i in 0..start.len() {
                            ctx.diagnostic(miette!(
                                labels = vec![LabeledSpan::at(
                                    (
                                        usize::try_from(start[i])
                                            .expect("for start of span to fit in usize"),
                                        usize::try_from(end[i] - start[i])
                                            .expect("for length of span to fit in usize")
                                    ),
                                    input_query.reason.as_str()
                                )],
                                "Unexpected error"
                            ));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use oxc_allocator::Allocator;
    use oxc_diagnostics::Report;
    use oxc_linter::LintContext;
    use oxc_parser::Parser;
    use oxc_query::schema;
    use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
    use oxc_span::SourceType;

    use super::{LinterPlugin, SingleTest};

    #[test]
    fn test_queries() {
        let plugin = LinterPlugin::new(schema(), "examples/queries");

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
            SemanticBuilder::new(source_text, source_type)
                .with_trivias(&ret.trivias)
                .build(program);

        assert!(
            errors.is_empty(),
            "In test {rule_name}: Semantic errors: {:?} Code:\n\n{:?}",
            errors,
            test.code
        );

        let semantic = Rc::new(semantic);

        let mut lint_ctx = LintContext::new(&Rc::clone(&semantic));

        plugin.run(
            &mut lint_ctx,
            Rc::clone(&semantic),
            test.relative_path.iter().map(|el| Some(el.clone())).collect::<Vec<_>>(),
            rule_name,
        );

        lint_ctx.into_message().into_iter().map(|m| m.error).collect::<Vec<_>>()
    }
}
