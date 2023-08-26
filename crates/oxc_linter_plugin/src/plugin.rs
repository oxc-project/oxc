use std::{collections::BTreeMap, fmt::Debug, fs, path::PathBuf, rc::Rc, sync::Arc};

use ignore::Walk;
use located_yaml::{YamlElt, YamlLoader};
use miette::{NamedSource, SourceSpan};
use mini_v8::{Value, Values};
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
    Report,
};
use oxc_linter::{Fix, LintContext};
use oxc_parser::Parser;
use oxc_query::{schema, Adapter};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::{SourceType, Span};
use serde::Deserialize;
use trustfall::{execute_query, FieldValue, Schema, TransparentValue};

enum SpanInfo {
    SingleSpanInfo(SingleSpanInfo),
    MultipleSpanInfo(MultipleSpanInfo),
}

#[derive(Debug, Deserialize)]
struct SingleSpanInfo {
    span_start: u64,
    span_end: u64,
    fix: Option<String>,
    summary: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MultipleSpanInfo {
    span_start: Vec<u64>,
    span_end: Vec<u64>,
    fix: Vec<Option<String>>,
    summary: Vec<Option<String>>,
    reason: Vec<Option<String>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct InputQuery {
    pub name: String,
    pub query: String,
    pub args: BTreeMap<Arc<str>, TransparentValue>,
    pub summary: String,
    pub reason: String,
    #[serde(skip_deserializing)]
    pub path: PathBuf,
    pub post_transform: Option<String>,
    #[serde(default)]
    pub tests: QueryTests,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub struct QueryTests {
    pub pass: Vec<SingleTest>,
    pub fail: Vec<SingleTest>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SingleTest {
    pub relative_path: Vec<String>,
    pub code: String,
}

#[derive(Debug)]
pub struct LinterPlugin {
    pub(super) rules: Vec<InputQuery>,
    schema: &'static Schema,
}

pub enum RulesToRun {
    All,
    Only(String),
}

#[derive(Debug, Error, Diagnostic)]
#[error(transparent)]
pub struct ParseError(serde_yaml::Error);

#[derive(Debug, Error, Diagnostic)]
pub enum ErrorFromLinterPlugin {
    #[error("{0}")]
    PluginGenerated(String, String, #[label("{1}")] Span),
    #[error("{error_message}")]
    Trustfall {
        /// Keep this name in sync with the `.contains()` and doc comment in [`self::test_queries`]
        error_message: String,
        #[source_code]
        query_source: Arc<NamedSource>,
        #[label = "This query failed."]
        query_span: SourceSpan,
    },
    #[error(transparent)]
    Ignore(ignore::Error),
    #[error(transparent)]
    ReadFile(std::io::Error),
    #[error("Failed to parse file at path: {0}")]
    QueryParse(PathBuf, #[related] Vec<ParseError>),
    #[error(
        "Expected span_start and span_end to be List of Int or Int, instead got\nspan_start = {span_start}\nspan_end = {span_end}"
    )]
    WrongTypeForSpanStartSpanEnd {
        span_start: String,
        span_end: String,
        #[source_code]
        query_source: Arc<NamedSource>,
        #[label = "This query failed."]
        query_span: SourceSpan,
    },
}

impl LinterPlugin {
    pub fn new(schema: &'static Schema, queries_path: &PathBuf) -> oxc_diagnostics::Result<Self> {
        let mut deserialized_queries = vec![];

        for dir_entry_found_maybe in Walk::new(queries_path) {
            let dir_entry_found = dir_entry_found_maybe.map_err(ErrorFromLinterPlugin::Ignore)?;

            let pathbuf = dir_entry_found.path().to_path_buf();

            if pathbuf.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("yml")) {
                let text = fs::read_to_string(&pathbuf).map_err(ErrorFromLinterPlugin::ReadFile)?;

                let mut deserialized =
                    serde_yaml::from_str::<InputQuery>(&text).map_err(|err| {
                        ErrorFromLinterPlugin::QueryParse(pathbuf.clone(), vec![ParseError(err)])
                    })?;
                deserialized.path = pathbuf;
                deserialized_queries.push(deserialized);
            }
        }

        Ok(Self { rules: deserialized_queries, schema })
    }

    #[allow(clippy::too_many_lines)]
    pub fn run_plugin_rules(
        &self,
        ctx: &mut LintContext,
        plugin: &InputQuery,
        adapter: &Adapter<'_>,
        mv8: &mini_v8::MiniV8,
    ) -> oxc_diagnostics::Result<()> {
        let arc_adapter = Arc::new(adapter);
        let query_source =
            Arc::new(NamedSource::new(plugin.path.to_string_lossy(), plugin.query.clone()));
        // NOTE: the 0 is technically wrong, but it's the right file which is enough
        let query_span = SourceSpan::new(0.into(), plugin.query.len().into());

        for data_item in
            execute_query(self.schema, Arc::clone(&arc_adapter), &plugin.query, plugin.args.clone())
                .map_err(|err| ErrorFromLinterPlugin::Trustfall {
                    error_message: err.to_string(),
                    query_source: Arc::clone(&query_source),
                    query_span,
                })?
        {
            let transformer: mini_v8::Function = mv8
                .eval(plugin.post_transform.clone().unwrap_or_else(|| "(data) => data".to_string()))
                .unwrap_or_else(|err| panic!("{err}"));

            let data = mv8.create_object();

            for (k, v) in data_item.clone() {
                fn to_v8(mv8: &mini_v8::MiniV8, fv: FieldValue) -> Value {
                    match fv {
                        FieldValue::Null => Value::Null,
                        FieldValue::Boolean(b) => Value::Boolean(b),
                        FieldValue::Int64(int) => {
                            let as_i32: i32 = int
                                .try_into()
                                .expect("for int64 number from trustfall to fit into i32 for js");
                            Value::Number(as_i32.into())
                        }
                        FieldValue::Uint64(int) => {
                            let as_u32: u32 = int
                                .try_into()
                                .expect("for Uint64 number from trustfall to fit into u32 for js");
                            Value::Number(as_u32.into())
                        }
                        FieldValue::Float64(f64) => Value::Number(f64),
                        FieldValue::String(str) => Value::String(mv8.create_string(&str)),
                        FieldValue::List(list) => {
                            let arr = mv8.create_array();
                            for el in &*list {
                                arr.push(to_v8(mv8, el.clone())).expect(
                                    "to be able to put elements from trustfall list into js list",
                                );
                            }
                            Value::Array(arr)
                        }
                        _ => unreachable!(),
                    }
                }
                data.set(k.to_string(), to_v8(mv8, v)).unwrap();
            }

            let fn_return: Value =
                transformer.call(Values::from_vec(vec![Value::Object(data)])).unwrap();

            let object_returned = match fn_return {
                Value::Null => continue,
                Value::Object(object) => object,
                _ => unimplemented!(
                    "You should not return values that aren't objects or nulls from js."
                ),
            };

            let transformed_data_to_span: SpanInfo = match (
                object_returned.get("span_start").unwrap(),
                object_returned.get("span_end").unwrap(),
            ) {
                (Value::Number(a), Value::Number(b)) => SpanInfo::SingleSpanInfo(SingleSpanInfo {
                    span_start: a as u64,
                    span_end: b as u64,
                    fix: match object_returned.get("fix").unwrap() {
                        Value::String(fix) => Some(fix.to_string()),
                        _ => None,
                    },
                    summary: match object_returned.get("summary").unwrap() {
                        Value::String(summary) => Some(summary.to_string()),
                        _ => None,
                    },
                    reason: match object_returned.get("reason").unwrap() {
                        Value::String(reason) => Some(reason.to_string()),
                        _ => None,
                    },
                }),
                (Value::Array(a), Value::Array(b)) => {
                    let a_count = a.len();
                    SpanInfo::MultipleSpanInfo(MultipleSpanInfo {
                        span_start: a
                            .elements()
                            .map(std::result::Result::unwrap)
                            .collect::<Vec<u64>>(),
                        span_end: b
                            .elements()
                            .map(std::result::Result::unwrap)
                            .collect::<Vec<u64>>(),
                        fix: match object_returned.get("fix").unwrap() {
                            Value::Array(fix_array) => (0..a_count)
                                .map(|i| match fix_array.get(i).unwrap() {
                                    Value::String(fix) => Some(fix.to_string()),
                                    _ => None,
                                })
                                .collect(),
                            _ => Vec::with_capacity(a_count as usize),
                        },
                        summary: match object_returned.get("summary").unwrap() {
                            Value::Array(summary_array) => (0..a_count)
                                .map(|i| match summary_array.get(i).unwrap() {
                                    Value::String(summary) => Some(summary.to_string()),
                                    _ => None,
                                })
                                .collect(),
                            _ => Vec::with_capacity(a_count as usize),
                        },
                        reason: match object_returned.get("reason").unwrap() {
                            Value::Array(reason_array) => (0..a_count)
                                .map(|i| match reason_array.get(i).unwrap() {
                                    Value::String(reason) => Some(reason.to_string()),
                                    _ => None,
                                })
                                .collect(),
                            _ => Vec::with_capacity(a_count as usize),
                        },
                    })
                }
                (a, b) => {
                    return Err(ErrorFromLinterPlugin::WrongTypeForSpanStartSpanEnd {
                        span_start: format!("{a:?}"),
                        span_end: format!("{b:?}"),
                        query_source: Arc::clone(&query_source),
                        query_span,
                    }
                    .into())
                }
            };

            ctx.with_rule_name(""); // leave this empty as it's a static string so we can't make it at runtime, and it's not userfacing

            match transformed_data_to_span {
                SpanInfo::SingleSpanInfo(SingleSpanInfo {
                    span_start: start,
                    span_end: end,
                    fix,
                    summary,
                    reason,
                }) => {
                    let start =
                        start.try_into().expect("Int64 or Uint64 of span_start to fit in u32");
                    let end = end.try_into().expect("Int64 or Uint64 of span_end to fit in u32");

                    let span = Span::new(start, end);

                    let error = ErrorFromLinterPlugin::PluginGenerated(
                        summary.unwrap_or_else(|| plugin.summary.clone()),
                        reason.unwrap_or_else(|| plugin.reason.clone()),
                        span,
                    );

                    if let Some(fix) = fix.map(|fixed_txt| Fix::new(fixed_txt, span)) {
                        ctx.diagnostic_with_fix(error, || fix);
                    } else {
                        ctx.diagnostic(error);
                    }
                }
                SpanInfo::MultipleSpanInfo(MultipleSpanInfo {
                    span_start: start,
                    span_end: end,
                    mut fix,
                    summary,
                    reason,
                }) => {
                    for i in 0..start.len() {
                        let start = start[i]
                            .try_into()
                            .expect("Int64 or Uint64 of span_start to fit in u32");
                        let end =
                            end[i].try_into().expect("Int64 or Uint64 of span_end to fit in u32");

                        let span = Span::new(start, end);

                        let error = ErrorFromLinterPlugin::PluginGenerated(
                            summary[i].clone().unwrap_or_else(|| plugin.summary.clone()),
                            reason[i].clone().unwrap_or_else(|| plugin.reason.clone()),
                            span,
                        );

                        if let Some(fix) = fix[i].take().map(|fixed_txt| Fix::new(fixed_txt, span))
                        {
                            ctx.diagnostic_with_fix(error, || fix);
                        } else {
                            ctx.diagnostic(error);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn run_tests(
        &self,
        ctx: &mut LintContext,
        relative_file_path_parts: Vec<Option<String>>,
        rules_to_run: RulesToRun,
        mini_v8: &mini_v8::MiniV8,
    ) -> oxc_diagnostics::Result<()> {
        let inner = Adapter::new(Rc::clone(ctx.semantic()), relative_file_path_parts);
        let adapter = Arc::from(&inner);
        if let RulesToRun::Only(this_rule) = rules_to_run {
            for rule in self.rules.iter().filter(|x| x.name == this_rule) {
                self.run_plugin_rules(ctx, rule, &adapter, mini_v8)?;
            }
        } else {
            for rule in &self.rules {
                self.run_plugin_rules(ctx, rule, &adapter, mini_v8)?;
            }
        }
        Ok(())
    }
}

fn run_test(
    test: &SingleTest,
    rule_name: &str,
    plugin: &LinterPlugin,
    mini_v8: &mini_v8::MiniV8,
) -> std::result::Result<Vec<Report>, Vec<Report>> {
    let file_path = &test.relative_path.last().expect("there to be atleast 1 path part");
    let source_text = &test.code;

    let allocator = Allocator::default();
    let source_type = SourceType::from_path(file_path).unwrap();
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    // Handle parser errors

    if !ret.errors.is_empty() {
        return Err(ret.errors);
    }

    let program = allocator.alloc(ret.program);
    let SemanticBuilderReturn { semantic, errors } =
        SemanticBuilder::new(source_text, source_type).with_trivias(ret.trivias).build(program);

    if !errors.is_empty() {
        return Err(errors);
    }

    let semantic = Rc::new(semantic);

    let mut lint_ctx = LintContext::new(&Rc::clone(&semantic));

    let result = plugin.run_tests(
        &mut lint_ctx,
        test.relative_path.iter().map(|el| Some(el.clone())).collect::<Vec<_>>(),
        RulesToRun::Only(rule_name.to_string()),
        mini_v8,
    );

    if let Some(err) = result.err() {
        return Err(vec![err]);
    }

    Ok(lint_ctx.into_message().into_iter().map(|m| m.error).collect::<Vec<_>>())
}

#[derive(Debug, Error, Diagnostic)]
#[error("Test expected to pass, but failed.")]
struct ExpectedTestToPassButFailed {
    #[source_code]
    query: NamedSource,
    #[label = "This test failed."]
    err_span: SourceSpan,
    #[related]
    errors: Vec<Report>,
}

#[derive(Debug, Error, Diagnostic)]
#[error("Test expected to fail, but passed.")]
struct ExpectedTestToFailButPassed {
    #[source_code]
    query: NamedSource,
    #[label = "This test should have failed but it passed."]
    err_span: SourceSpan,
}

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected errors in fail test.")]
struct UnexpectedErrorsInFailTest {
    #[related]
    errors: Vec<Report>,
    #[source_code]
    query: NamedSource,
    #[label = "This test should have failed but it passed."]
    err_span: SourceSpan,
}

enum PassOrFail {
    Pass,
    Fail,
}

fn span_of_test_n(
    yaml_text: &str,
    test_ix: usize,
    test_code: &str,
    pass_or_fail: &PassOrFail,
) -> SourceSpan {
    let yaml = YamlLoader::load_from_str(
        // TODO: Should we just save the string after we read it originally?
        yaml_text,
    )
    .expect("to be able to parse yaml for error reporting");
    let YamlElt::Hash(hash) = &yaml.docs[0].yaml else {
        unreachable!("must be a top level hashmap in the yaml")
    };
    let tests_hash_key = hash
        .keys()
        .find(|x| {
            let YamlElt::String(str) = &x.yaml else { return false };
            str == "tests"
        })
        .expect("to be able to find tests hash in yaml file");
    let YamlElt::Hash(tests_hash) = &hash[tests_hash_key].yaml else {
        unreachable!("there must be a tests hashmap in the yaml")
    };
    let pass_hash_key = tests_hash
        .keys()
        .find(|x| {
            let YamlElt::String(str) = &x.yaml else { return false };
            str == if matches!(pass_or_fail, PassOrFail::Pass) { "pass" } else { "fail" }
        })
        .expect("to be able to find pass hash in yaml file");
    let YamlElt::Array(test_arr) = &tests_hash[pass_hash_key].yaml else {
        unreachable!("there must be a pass array in the yaml")
    };
    let test_hash_span = &test_arr[test_ix].lines_range();
    let start = yaml_text
        .char_indices()
        .filter(|a| a.1 == '\n')
        .nth(test_hash_span.0 - 1) // subtract one because span is 1-based
        .map(|a| a.0)
        .expect("to find start of span of test");
    let start_of_end = yaml_text[start..]
        .find(&test_code[0..test_code.find('\n').unwrap_or(test_code.len())])
        .expect("to find start of end")
        + start;

    let nl = test_code.chars().filter(|a| *a == '\n').count();
    let end_of_end = yaml_text[start_of_end..]
        .char_indices()
        .filter(|a| a.1 == '\n')
        .nth(nl - 1)
        .map(|a| a.0)
        .expect("to find end of end of span of test")
        + start_of_end;

    SourceSpan::new(start.into(), (end_of_end - start).into())
}

pub fn test_queries(queries_to_test: &PathBuf) -> oxc_diagnostics::Result<()> {
    let plugin = LinterPlugin::new(schema(), queries_to_test)?;
    let mini_v8 = mini_v8::MiniV8::new();

    for rule in &plugin.rules {
        for (ix, test) in rule.tests.pass.iter().enumerate() {
            let diagnostics_collected = run_test(test, &rule.name, &plugin, &mini_v8);
            let source = Arc::new(NamedSource::new(
                format!("./{}", test.relative_path.join("/")),
                test.code.clone(),
            ));

            match diagnostics_collected {
                Err(errs) | Ok(errs) if !errs.is_empty() => {
                    let yaml_text =
                        fs::read_to_string(&rule.path).expect("to be able to get text of rule");

                    return Err(ExpectedTestToPassButFailed {
                        errors: errs
                            .into_iter()
                            .map(|e| {
                                // Don't change the sourcecode of errors that already have their own sourcecode
                                if e.source_code().is_some() {
                                    e
                                } else {
                                    e.with_source_code(Arc::clone(&source))
                                }
                            })
                            .collect(),
                        err_span: span_of_test_n(&yaml_text, ix, &test.code, &PassOrFail::Pass),
                        query: NamedSource::new(rule.path.to_string_lossy(), yaml_text),
                    }
                    .into());
                }
                _ => {}
            };
        }

        for (i, test) in rule.tests.fail.iter().enumerate() {
            let diagnostics_collected = run_test(test, &rule.name, &plugin, &mini_v8);
            let source = Arc::new(NamedSource::new(
                format!("./{}", test.relative_path.join("/")),
                test.code.clone(),
            ));

            match diagnostics_collected {
                Ok(errs)
                    if errs.len() == 1
                        && format!("{:#?}", errs[0]).starts_with("PluginGenerated") =>
                { /* Success case. */ }
                Ok(errs) if errs.is_empty() => {
                    let yaml_text =
                        fs::read_to_string(&rule.path).expect("to be able to get text of rule");

                    return Err(ExpectedTestToFailButPassed {
                        err_span: span_of_test_n(&yaml_text, i, &test.code, &PassOrFail::Fail),
                        query: NamedSource::new(rule.path.to_string_lossy(), yaml_text),
                    }
                    .into());
                }
                Err(errs) | Ok(errs) => {
                    let yaml_text =
                        fs::read_to_string(&rule.path).expect("to be able to get text of rule");

                    return Err(UnexpectedErrorsInFailTest {
                        errors: errs
                            .into_iter()
                            .map(|e| {
                                // Don't change the sourcecode of errors that already have their own sourcecode
                                if e.source_code().is_some() {
                                    e
                                } else {
                                    e.with_source_code(Arc::clone(&source))
                                }
                            })
                            .collect(),
                        err_span: span_of_test_n(&yaml_text, i, &test.code, &PassOrFail::Fail),
                        query: NamedSource::new(rule.path.to_string_lossy(), yaml_text),
                    }
                    .into());
                }
            }
        }

        if rule.tests.pass.len() + rule.tests.fail.len() > 0 {
            println!(
                "{} passed {} tests successfully.\n",
                rule.name,
                rule.tests.pass.len() + rule.tests.fail.len()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::test_queries;

    #[test]
    fn query_tests() -> oxc_diagnostics::Result<()> {
        test_queries(&Path::new("examples/queries").to_path_buf())?;
        Ok(())
    }
}
