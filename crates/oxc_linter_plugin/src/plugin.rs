use std::{collections::BTreeMap, fmt::Debug, fs, path::PathBuf, rc::Rc, sync::Arc};

use ignore::Walk;
use miette::{NamedSource, SourceSpan};
use oxc_allocator::Allocator;
use oxc_diagnostics::{
    miette::{self},
    Report,
};
use oxc_linter::LintContext;
use oxc_parser::Parser;
use oxc_query::{schema, Adapter};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::SourceType;
use serde::Deserialize;
use trustfall::{execute_query, FieldValue, Schema, TransparentValue};

use crate::{
    errors::{
        ErrorFromLinterPlugin, ExpectedTestToFailButPassed, ExpectedTestToPassButFailed,
        SpanStartOrEnd, UnexpectedErrorsInFailTest,
    },
    raw_diagnostic::RawPluginDiagnostic,
    spans::{span_of_test_n, PassOrFail},
};

/// Represents a single parsed yaml plugin file. Includes
/// the query, tests, and metadata about the query.
#[derive(Deserialize, Clone, Debug)]
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

/// Represents all of the tests for a plugin.
#[derive(Deserialize, Default, Clone, Debug)]
pub struct QueryTests {
    pub pass: Vec<SingleTest>,
    pub fail: Vec<SingleTest>,
}

/// Represents a single test for a plugin.
#[derive(Deserialize, Clone, Debug)]
pub struct SingleTest {
    pub relative_path: Vec<String>,
    pub code: String,
}

/// Holds multiple parsed rules.
#[derive(Debug)]
pub struct LinterPlugin {
    rules: Vec<InputQuery>,
    schema: &'static Schema,
}

/// Whether to run all rules or only a specific rule.
#[allow(dead_code)]
pub enum RulesToRun {
    /// Execute all rules.
    All,
    /// The rules to run will be the rules whose names are equal
    /// to the string stored in the variant.
    Only(String),
}

impl LinterPlugin {
    /// Parses all queries in the directory provided, going down into nested directories looking for .yml files.
    ///
    /// # Errors
    /// This function will error if it can't read a file, or if it can't parse a query
    pub fn new(schema: &'static Schema, queries_path: &PathBuf) -> oxc_diagnostics::Result<Self> {
        let mut deserialized_queries = vec![];

        for dir_entry_found_maybe in Walk::new(queries_path) {
            let dir_entry_found = dir_entry_found_maybe.map_err(ErrorFromLinterPlugin::Ignore)?;

            let pathbuf = dir_entry_found.path().to_path_buf();

            if pathbuf.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("yml")) {
                let text = fs::read_to_string(&pathbuf).map_err(ErrorFromLinterPlugin::ReadFile)?;

                let mut deserialized =
                    serde_yaml::from_str::<InputQuery>(&text).map_err(|err| {
                        ErrorFromLinterPlugin::QueryParse(pathbuf.clone(), vec![err.into()])
                    })?;
                deserialized.path = pathbuf;
                deserialized_queries.push(deserialized);
            }
        }

        Ok(Self { rules: deserialized_queries, schema })
    }

    /// Run specific plugin rule by reference on parsed code.
    ///
    /// # Errors
    /// If the query fails to execute, if the query's output types are wrong, or if query
    /// execution has an error.
    //
    // allow clippy::redundant_allocation which is upset we need the Arc<&'_ Adapter>
    // and not a Arc<Adapter> however we need the reference to be arc'd because
    // the Adapter trait is implemented for a &Adapter, not just Adapter
    #[allow(clippy::redundant_allocation)]
    fn run_specific_plugin_rule(
        &self,
        ctx: &mut LintContext,
        plugin: &InputQuery,
        adapter: &Arc<&Adapter<'_>>,
    ) -> oxc_diagnostics::Result<()> {
        let query_source =
            Arc::new(NamedSource::new(plugin.path.to_string_lossy(), plugin.query.clone()));
        // NOTE: the 0 is technically wrong, but it's the right file which is enough
        //       the reason we lie about the span is because in the NamedSource, we just put
        //       the query rather than the real yaml file, and this error is about the whole
        //       query, so we just highlight the entire length of the query. This of course is
        //       results in the wrong row when the user ctrl-clicks, but it saves us from having
        //       to reparse the whole yaml file in order to get the spans.
        let query_span = SourceSpan::new(0.into(), plugin.query.len().into());

        let query_results =
            execute_query(self.schema, Arc::clone(adapter), &plugin.query, plugin.args.clone())
                .map_err(|err| ErrorFromLinterPlugin::Trustfall {
                    error_message: err.to_string(),
                    query_source: Arc::clone(&query_source),
                    query_span,
                })?;

        for result in query_results {
            let transformed_data_to_span = match (result.get("span_start"), result.get("span_end"))
            {
                (Some(FieldValue::Uint64(span_start)), Some(FieldValue::Uint64(span_end))) => {
                    let transformed: Result<RawPluginDiagnostic, SpanStartOrEnd> =
                        (*span_start, *span_end).try_into();

                    vec![transformed.map_err(|which_span| {
                        ErrorFromLinterPlugin::SpanStartOrEndDoesntFitInU32 {
                            number: match which_span {
                                SpanStartOrEnd::Start => *span_start,
                                SpanStartOrEnd::End => *span_end,
                            }
                            .into(),
                            query_span,
                            which_span,
                            query_source: Arc::clone(&query_source),
                        }
                    })?]
                }
                (Some(FieldValue::Int64(span_start)), Some(FieldValue::Int64(span_end))) => {
                    let transformed: Result<RawPluginDiagnostic, SpanStartOrEnd> =
                        (*span_start, *span_end).try_into();

                    vec![transformed.map_err(|which_span| {
                        ErrorFromLinterPlugin::SpanStartOrEndDoesntFitInU32 {
                            number: match which_span {
                                SpanStartOrEnd::Start => *span_start,
                                SpanStartOrEnd::End => *span_end,
                            }
                            .into(),
                            query_span,
                            which_span,
                            query_source: Arc::clone(&query_source),
                        }
                    })?]
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

            for plugin_diagnostic in transformed_data_to_span {
                let error = ErrorFromLinterPlugin::PluginGenerated(
                    plugin.summary.clone(),
                    plugin.reason.clone(),
                    plugin_diagnostic.into(),
                );

                ctx.diagnostic(error);
            }
        }
        Ok(())
    }

    /// Run specific plugin rule by name or multiple plugin rules on parsed code.
    ///
    /// # Errors
    /// Any errors that occur while linting the file, such as if the file can't be read,
    /// or if the file can't be parsed, or if the query can't be executed, or if the query's
    /// output types are wrong.
    pub fn lint_file(
        &self,
        ctx: &mut LintContext,
        relative_file_path_parts: Vec<Option<String>>,
        rules_to_run: RulesToRun,
    ) -> oxc_diagnostics::Result<()> {
        let inner = Adapter::new(Rc::clone(ctx.semantic()), relative_file_path_parts);
        let adapter = Arc::from(&inner);
        if let RulesToRun::Only(this_rule) = rules_to_run {
            for rule in self.rules.iter().filter(|x| x.name == this_rule) {
                self.run_specific_plugin_rule(ctx, rule, &adapter)?;
            }
        } else {
            for rule in &self.rules {
                self.run_specific_plugin_rule(ctx, rule, &adapter)?;
            }
        }
        Ok(())
    }
}

/// Run one individual test on unparsed code.
fn run_individual_test(
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

    // Handle semantic errors
    if !errors.is_empty() {
        return Err(errors);
    }

    let semantic = Rc::new(semantic);

    let mut lint_ctx = LintContext::new(&Rc::clone(&semantic));

    let result = plugin.lint_file(
        &mut lint_ctx,
        test.relative_path.iter().map(|el| Some(el.clone())).collect::<Vec<_>>(),
        RulesToRun::Only(rule_name.to_string()),
    );

    // Handle query errors
    if let Some(err) = result.err() {
        return Err(vec![err]);
    }

    // Return plugin made errors
    Ok(lint_ctx.into_message().into_iter().map(|m| m.error).collect::<Vec<_>>())
}

/// Enumerates and tests all queries at the path given.
/// # Errors
/// Unable to read any of the yaml rule files or unable to parse any of the yaml rule files,
/// or if any test expected to pass but failed, or if any test expected to fail but passed,
/// or query execution errors such as if the `span_start` and `span_end` are not both
/// understood types by the error reporting system.
pub fn test_queries(queries_to_test: &PathBuf) -> oxc_diagnostics::Result<()> {
    let plugin = LinterPlugin::new(schema(), queries_to_test)?;

    for rule in &plugin.rules {
        for (ix, test) in rule.tests.pass.iter().enumerate() {
            let diagnostics_collected = run_individual_test(test, &rule.name, &plugin);
            let source = Arc::new(NamedSource::new(
                format!("./{}", test.relative_path.join("/")),
                test.code.clone(),
            ));

            match diagnostics_collected {
                Err(errs) | Ok(errs) if !errs.is_empty() => {
                    let yaml_text =
                        fs::read_to_string(&rule.path).map_err(ErrorFromLinterPlugin::ReadFile)?;

                    let errors_with_code = errs
                        .into_iter()
                        .map(|e| {
                            // Don't change the sourcecode of errors that already have their own sourcecode
                            if e.source_code().is_some() {
                                e
                            } else {
                                // Add js code to errors that don't have code yet
                                e.with_source_code(Arc::clone(&source))
                            }
                        })
                        .collect();

                    return Err(ExpectedTestToPassButFailed {
                        errors: errors_with_code,
                        err_span: span_of_test_n(&yaml_text, ix, &test.code, &PassOrFail::Pass),
                        query: NamedSource::new(rule.path.to_string_lossy(), yaml_text),
                    }
                    .into());
                }
                _ => { /* Ignore the empty diagnostics, as it means the test passed. */ }
            };
        }

        for (i, test) in rule.tests.fail.iter().enumerate() {
            let diagnostics_collected = run_individual_test(test, &rule.name, &plugin);
            let source = Arc::new(NamedSource::new(
                format!("./{}", test.relative_path.join("/")),
                test.code.clone(),
            ));

            match diagnostics_collected {
                Ok(errs)
                    if errs.len() == 1 // TODO: Handle more than one error
                        && matches!(
                            errs[0].downcast_ref::<ErrorFromLinterPlugin>(),
                            Some(ErrorFromLinterPlugin::PluginGenerated(..))
                        ) =>
                { /* Success case. */ }
                Ok(errs) if errs.is_empty() => {
                    let yaml_text =
                        fs::read_to_string(&rule.path).map_err(ErrorFromLinterPlugin::ReadFile)?;

                    return Err(ExpectedTestToFailButPassed {
                        err_span: span_of_test_n(&yaml_text, i, &test.code, &PassOrFail::Fail),
                        query: NamedSource::new(rule.path.to_string_lossy(), yaml_text),
                    }
                    .into());
                }
                Err(errs) | Ok(errs) => {
                    let yaml_text =
                        fs::read_to_string(&rule.path).map_err(ErrorFromLinterPlugin::ReadFile)?;

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
