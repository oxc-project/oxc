use std::{collections::BTreeMap, fmt::Debug, fs, path::PathBuf, rc::Rc, sync::Arc};

use crate::{
    errors::{ErrorFromLinterPlugin, SpanStartOrEnd},
    js::trustfall_results_to_js_arguments,
    raw_diagnostic::RawPluginDiagnostic,
};
use ignore::Walk;
use miette::{NamedSource, SourceSpan};
use mini_v8::{MiniV8, Value};
use oxc_diagnostics::miette::{self};
use oxc_linter::LintContext;
use oxc_query::{schema, Adapter};
use serde::Deserialize;
use trustfall::{execute_query, FieldValue, TransparentValue};

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
    pub post_transform: Option<String>,
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
    pub(crate) rules: Vec<InputQuery>,
}

impl LinterPlugin {
    /// Parses all queries in the directory provided, going down into nested directories looking for .yml files.
    ///
    /// # Errors
    /// This function will error if it can't read a file, or if it can't parse a query
    pub fn new(queries_path: &PathBuf) -> oxc_diagnostics::Result<Self> {
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

        Ok(Self { rules: deserialized_queries })
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
            execute_query(schema(), Arc::clone(adapter), &plugin.query, plugin.args.clone())
                .map_err(|err| ErrorFromLinterPlugin::Trustfall {
                    error_message: err.to_string(),
                    query_source: Arc::clone(&query_source),
                    query_span,
                })?;

        let mv8 = MiniV8::new();

        for result in query_results {
            let transformed_data_to_span = if let Some(post_transform_code) = &plugin.post_transform
            {
                let transformer: mini_v8::Function =
                    mv8.eval(&**post_transform_code).unwrap_or_else(|err| panic!("{err}"));

                let arguments = trustfall_results_to_js_arguments(&mv8, result);

                let fn_return: Value =
                    transformer.call(arguments).unwrap_or_else(|err| panic!("{err}"));

                let object_returned = match fn_return {
                    Value::Null => continue,
                    Value::Object(object) => object,
                    returned => {
                        return Err(ErrorFromLinterPlugin::UnexpectedJSReturn {
                            what_was_returned: format!("{returned:#?}"),
                            post_transform_code: Arc::new(NamedSource::new(
                                plugin.path.to_string_lossy(),
                                post_transform_code.clone(),
                            )),
                            // NOTE: the 0 is technically wrong, but it's the right file which is enough
                            //       the reason we lie about the span is because in the NamedSource, we just put
                            //       the query rather than the real yaml file, and this error is about the whole
                            //       query, so we just highlight the entire length of the query. This of course is
                            //       results in the wrong row when the user ctrl-clicks, but it saves us from having
                            //       to reparse the whole yaml file in order to get the spans.
                            post_transform_span: SourceSpan::new(
                                0.into(),
                                post_transform_code.len().into(),
                            ),
                        }
                        .into());
                    }
                };
                Self::decode_js_span_start_end(
                    object_returned
                        .get::<_, Option<u64>>("span_start")
                        .expect("to be able to get the span_start field from the object"),
                    object_returned
                        .get::<_, Option<u64>>("span_end")
                        .expect("to be able to get the span_end field from the object"),
                    Arc::clone(&query_source),
                    query_span,
                )?
            } else {
                Self::decode_trustfall_span_start_end(
                    result.get("span_start"),
                    result.get("span_end"),
                    Arc::clone(&query_source),
                    query_span,
                )?
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

    fn decode_trustfall_span_start_end(
        span_start: Option<&FieldValue>,
        span_end: Option<&FieldValue>,
        query_source: Arc<NamedSource>,
        query_span: SourceSpan,
    ) -> oxc_diagnostics::Result<Vec<RawPluginDiagnostic>> {
        match (span_start, span_end) {
            (Some(FieldValue::Uint64(span_start)), Some(FieldValue::Uint64(span_end))) => {
                let transformed: Result<RawPluginDiagnostic, SpanStartOrEnd> =
                    (*span_start, *span_end).try_into();

                Ok(vec![transformed.map_err(|which_span| {
                    ErrorFromLinterPlugin::SpanStartOrEndDoesntFitInU32 {
                        number: match which_span {
                            SpanStartOrEnd::Start => *span_start,
                            SpanStartOrEnd::End => *span_end,
                        }
                        .into(),
                        query_span,
                        which_span,
                        query_source,
                    }
                })?])
            }
            (Some(FieldValue::Int64(span_start)), Some(FieldValue::Int64(span_end))) => {
                let transformed: Result<RawPluginDiagnostic, SpanStartOrEnd> =
                    (*span_start, *span_end).try_into();

                Ok(vec![transformed.map_err(|which_span| {
                    ErrorFromLinterPlugin::SpanStartOrEndDoesntFitInU32 {
                        number: match which_span {
                            SpanStartOrEnd::Start => *span_start,
                            SpanStartOrEnd::End => *span_end,
                        }
                        .into(),
                        query_span,
                        which_span,
                        query_source,
                    }
                })?])
            }
            (a, b) => Err(ErrorFromLinterPlugin::WrongTypeForSpanStartSpanEnd {
                span_start: format!("{a:?}"),
                span_end: format!("{b:?}"),
                query_source,
                query_span,
            }
            .into()),
        }
    }

    fn decode_js_span_start_end(
        span_start: Option<u64>,
        span_end: Option<u64>,
        query_source: Arc<NamedSource>,
        query_span: SourceSpan,
    ) -> oxc_diagnostics::Result<Vec<RawPluginDiagnostic>> {
        match (span_start, span_end) {
            (Some(span_start), Some(span_end)) => {
                let transformed: Result<RawPluginDiagnostic, SpanStartOrEnd> =
                    (span_start, span_end).try_into();

                Ok(vec![transformed.map_err(|which_span| {
                    ErrorFromLinterPlugin::SpanStartOrEndDoesntFitInU32 {
                        number: match which_span {
                            SpanStartOrEnd::Start => span_start,
                            SpanStartOrEnd::End => span_end,
                        }
                        .into(),
                        query_span: SourceSpan::new(0.into(), 0.into()),
                        which_span,
                        query_source,
                    }
                })?])
            }
            (a, b) => Err(ErrorFromLinterPlugin::WrongTypeForSpanStartSpanEnd {
                span_start: format!("{a:?}"),
                span_end: format!("{b:?}"),
                query_source,
                query_span,
            }
            .into()),
        }
    }

    /// Run all plugin rules on parsed code.
    ///
    /// # Errors
    /// Any errors that occur while linting the file, such as if the file can't be read,
    /// or if the file can't be parsed, or if the query can't be executed, or if the query's
    /// output types are wrong.
    pub fn lint_file(
        &self,
        ctx: &mut LintContext,
        relative_file_path_parts: Vec<Option<String>>,
    ) -> oxc_diagnostics::Result<()> {
        let inner = Adapter::new(Rc::clone(ctx.semantic()), relative_file_path_parts);
        let adapter = Arc::from(&inner);
        for rule in &self.rules {
            Self::run_specific_plugin_rule(ctx, rule, &adapter)?;
        }
        Ok(())
    }

    /// Run specific plugin rule by name rules on parsed code.
    ///
    /// # Errors
    /// Any errors that occur while linting the file, such as if the file can't be read,
    /// or if the file can't be parsed, or if the query can't be executed, or if the query's
    /// output types are wrong.
    #[cfg(test)]
    pub(crate) fn lint_file_with_rule(
        &self,
        ctx: &mut LintContext,
        relative_file_path_parts: Vec<Option<String>>,
        rule_name: &str,
    ) -> oxc_diagnostics::Result<()> {
        let inner = Adapter::new(Rc::clone(ctx.semantic()), relative_file_path_parts);
        let adapter = Arc::from(&inner);
        for rule in self.rules.iter().filter(|x| x.name == rule_name) {
            Self::run_specific_plugin_rule(ctx, rule, &adapter)?;
        }
        Ok(())
    }
}
