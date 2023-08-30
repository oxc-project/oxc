use std::path::Path;

use crate::{
    errors::{
        ExpectedTestToFailButPassed, ExpectedTestToPassButFailed, UnexpectedErrorsInFailTest,
    },
    spans::{span_of_test_n, PassOrFail},
};
use crate::{plugin::SingleTest, LinterPlugin};
use oxc_allocator::Allocator;
use oxc_diagnostics::Report;
use oxc_parser::Parser;
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::SourceType;
use std::path::PathBuf;

/// Run one individual test on unparsed code.
#[cfg(test)]
fn run_individual_test(
    test: &SingleTest,
    rule_name: &str,
    plugin: &LinterPlugin,
) -> std::result::Result<Vec<Report>, Vec<Report>> {
    use std::rc::Rc;

    use oxc_linter::LintContext;

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

    let result = plugin.lint_file_with_rule(
        &mut lint_ctx,
        test.relative_path.iter().map(|el| Some(el.clone())).collect::<Vec<_>>(),
        rule_name,
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
#[cfg(test)]
pub fn test_queries(queries_to_test: &PathBuf) -> oxc_diagnostics::Result<()> {
    use std::{fs, sync::Arc};

    use miette::NamedSource;

    use crate::errors::ErrorFromLinterPlugin;

    let plugin = LinterPlugin::new(queries_to_test)?;

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

#[test]
fn query_tests() -> oxc_diagnostics::Result<()> {
    test_queries(&Path::new("examples/queries").to_path_buf())?;
    Ok(())
}
