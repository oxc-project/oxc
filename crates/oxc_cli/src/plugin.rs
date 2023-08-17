use std::{collections::BTreeMap, fs, path::PathBuf, rc::Rc, sync::Arc};

use ignore::Walk;
use located_yaml::{YamlElt, YamlLoader};
use miette::{NamedSource, SourceSpan};
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
    #[serde(skip_deserializing)]
    pub path: PathBuf,
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
        "Missing span_start and span_end in query output. Make sure you output the start and end of a span in the query under the names span_start and span_end."
    )]
    MissingSpanStartAndSpanEnd {
        #[source_code]
        query_source: Arc<NamedSource>,
        #[label = "This query failed."]
        query_span: SourceSpan,
    },
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
    pub fn new(schema: &'static Schema, queries_path: PathBuf) -> oxc_diagnostics::Result<Self> {
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

    pub fn run_plugin_rules(
        &self,
        ctx: &mut LintContext,
        plugin: &InputQuery,
        adapter: &Adapter<'_>,
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
            {
                let transformed_data_to_span =
                    match (data_item.get("span_start"), data_item.get("span_end")) {
                        (Some(FieldValue::List(x)), Some(FieldValue::List(y)))
                            if matches!(x[0], FieldValue::Int64(_))
                                && matches!(y[0], FieldValue::Int64(_)) =>
                        {
                            data_item
                                .try_into_struct::<MultipleSpanInfo>()
                                .map(SpanInfo::MultipleSpanInfo)
                                .expect("to be able to convert into MultipleSpanInfo")
                        }
                        (Some(FieldValue::List(x)), Some(FieldValue::List(y)))
                            if matches!(x[0], FieldValue::Uint64(_))
                                && matches!(y[0], FieldValue::Uint64(_)) =>
                        {
                            data_item
                                .try_into_struct::<MultipleSpanInfo>()
                                .map(SpanInfo::MultipleSpanInfo)
                                .expect("to be able to convert into MultipleSpanInfo")
                        }
                        (Some(FieldValue::Int64(_)), Some(FieldValue::Int64(_)))
                        | (Some(FieldValue::Uint64(_)), Some(FieldValue::Uint64(_))) => data_item
                            .try_into_struct::<SingleSpanInfo>()
                            .map(SpanInfo::SingleSpanInfo)
                            .expect("to be able to convert into SingleSpanInfo"),
                        (None, None) => {
                            return Err(ErrorFromLinterPlugin::MissingSpanStartAndSpanEnd {
                                query_source: Arc::clone(&query_source),
                                query_span,
                            }
                            .into())
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
                    }) => {
                        ctx.diagnostic(ErrorFromLinterPlugin::PluginGenerated(
                            plugin.summary.clone(),
                            plugin.reason.clone(),
                            Span {
                                start: start
                                    .try_into()
                                    .expect("Int64 or Uint64 of span_start to fit in u32"),
                                end: end
                                    .try_into()
                                    .expect("Int64 or Uint64 of span_end to fit in u32"),
                            },
                        ));
                    }
                    SpanInfo::MultipleSpanInfo(MultipleSpanInfo {
                        span_start: start,
                        span_end: end,
                    }) => {
                        for i in 0..start.len() {
                            ctx.diagnostic(ErrorFromLinterPlugin::PluginGenerated(
                                plugin.summary.clone(),
                                plugin.reason.clone(),
                                Span {
                                    start: start[i]
                                        .try_into()
                                        .expect("Int64 or Uint64 of span_start to fit in u32"),
                                    end: end[i]
                                        .try_into()
                                        .expect("Int64 or Uint64 of span_end to fit in u32"),
                                },
                            ));
                        }
                    }
                }
            };
        }
        Ok(())
    }

    pub fn run_tests(
        &self,
        ctx: &mut LintContext,
        relative_file_path_parts: Vec<Option<String>>,
        rules_to_run: RulesToRun,
    ) -> oxc_diagnostics::Result<()> {
        let inner = Adapter::new(Rc::clone(ctx.semantic()), relative_file_path_parts);
        let adapter = Arc::from(&inner);
        if let RulesToRun::Only(this_rule) = rules_to_run {
            for rule in self.rules.iter().filter(|x| x.name == this_rule) {
                self.run_plugin_rules(ctx, rule, &adapter)?;
            }
        } else {
            for rule in &self.rules {
                self.run_plugin_rules(ctx, rule, &adapter)?;
            }
        }
        Ok(())
    }
}

fn run_test(
    test: &SingleTest,
    rule_name: &str,
    plugin: &LinterPlugin,
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
}

fn span_of_test_n(query_text: &str, test_ix: usize) -> SourceSpan {
    let yaml = YamlLoader::load_from_str(
        // TODO: Should we just save the string after we read it originally?
        query_text,
    )
    .expect("to be able to parse yaml for error reporting");
    let YamlElt::Hash(hash) = &yaml.docs[0].yaml else {unreachable!("must be a top level hashmap in the yaml")};
    let tests_hash_key = hash
        .keys()
        .find(|x| {
            let YamlElt::String(str) = &x.yaml else {return false};
            str == "tests"
        })
        .expect("to be able to find tests hash in yaml file");
    let YamlElt::Hash(tests_hash) = &hash[tests_hash_key].yaml else {unreachable!("there must be a tests hashmap in the yaml")};
    let pass_hash_key = tests_hash
        .keys()
        .find(|x| {
            let YamlElt::String(str) = &x.yaml else {return false};
            str == "pass"
        })
        .expect("to be able to find pass hash in yaml file");
    let YamlElt::Array(passing_test_arr) = &tests_hash[pass_hash_key].yaml else {unreachable!("there must be a pass array in the yaml")};
    let test_hash_span = &passing_test_arr[test_ix].lines_range();
    let start = query_text
        .char_indices()
        .filter(|a| a.1 == '\n')
        .nth(test_hash_span.0 - 1) // subtract one because span is 1-based
        .map(|a| a.0)
        .expect("to find start of span of test");
    let end = query_text
        .char_indices()
        .filter(|a| a.1 == '\n')
        .nth(test_hash_span.1 - 1) // subtract one because span is 1-based
        .map(|a| a.0)
        .expect("to find start of span of test");
    SourceSpan::new(start.into(), (end - start).into())
}

pub fn test_queries(queries_to_test: PathBuf) -> oxc_diagnostics::Result<()> {
    let plugin = LinterPlugin::new(schema(), queries_to_test)?;

    for rule in &plugin.rules {
        for (ix, test) in rule.tests.pass.iter().enumerate() {
            let diagnostics_collected = run_test(test, &rule.name, &plugin);
            let source = Arc::new(NamedSource::new(
                format!("./{}", test.relative_path.join("/")),
                test.code.clone(),
            ));
            match diagnostics_collected {
                Err(errs) | Ok(errs) if !errs.is_empty() => {
                    let query_text =
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
                        err_span: span_of_test_n(&query_text, ix),
                        query: NamedSource::new(rule.path.to_string_lossy(), query_text),
                    }
                    .into());
                }
                _ => {}
            };
        }

        for (i, test) in rule.tests.fail.iter().enumerate() {
            let messages = run_test(test, &rule.name, &plugin);
            match messages {
                Ok(errs)
                    if errs.len() == 1
                        && format!("{:#?}", errs[0]).starts_with("PluginGenerated") =>
                { /* Success case. */ }
                Ok(errs) if errs.is_empty() => {
                    let query_text =
                        fs::read_to_string(&rule.path).expect("to be able to get text of rule");

                    return Err(ExpectedTestToFailButPassed {
                        err_span: span_of_test_n(&query_text, i),
                        query: NamedSource::new(rule.path.to_string_lossy(), query_text),
                    }
                    .into());
                }
                Err(errs) | Ok(errs) => {
                    return Err(UnexpectedErrorsInFailTest { errors: errs }.into())
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

    use miette::Result;

    use super::test_queries;

    #[test]
    fn query_tests() -> Result<()> {
        test_queries(Path::new("examples/queries").to_path_buf())?;
        Ok(())
    }
}
