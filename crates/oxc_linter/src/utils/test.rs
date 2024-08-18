use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::LintContext;

use super::{is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind, PossibleJestNode};

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedTestMethodsConfig {
    pub(crate) restricted_test_methods: FxHashMap<String, String>,
}

pub trait NoRestrictedTestMethods {
    fn restricted_test_methods(&self) -> &FxHashMap<String, String>;

    fn contains(&self, key: &str) -> bool {
        self.restricted_test_methods().contains_key(key)
    }

    fn get_message(&self, name: &str) -> Option<String> {
        self.restricted_test_methods().get(name).cloned()
    }

    fn get_configuration(value: &serde_json::Value) -> NoRestrictedTestMethodsConfig {
        let restricted_test_methods = value
            .get(0)
            .and_then(serde_json::Value::as_object)
            .and_then(Self::compile_restricted_jest_methods)
            .unwrap_or_default();

        NoRestrictedTestMethodsConfig { restricted_test_methods: restricted_test_methods.clone() }
    }

    fn run_test<'a>(
        &self,
        possible_test_node: &PossibleJestNode<'a, '_>,
        ctx: &LintContext<'a>,
        is_jest: bool,
    ) {
        let node = possible_test_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let kind = if is_jest { JestGeneralFnKind::Jest } else { JestGeneralFnKind::Vitest };

        if !is_type_of_jest_fn_call(
            call_expr,
            possible_test_node,
            ctx,
            &[JestFnKind::General(kind)],
        ) {
            return;
        }

        let Some(mem_expr) = call_expr.callee.as_member_expression() else {
            return;
        };
        let Some(property_name) = mem_expr.static_property_name() else {
            return;
        };
        let Some((span, _)) = mem_expr.static_property_info() else {
            return;
        };

        if self.contains(property_name) {
            let method_name = if is_jest { "jest" } else { "vi" };
            self.get_message(property_name).map_or_else(
                || {
                    ctx.diagnostic(Self::restricted_test_method(method_name, property_name, span));
                },
                |message| {
                    if message.trim() == "" {
                        ctx.diagnostic(Self::restricted_test_method(
                            method_name,
                            property_name,
                            span,
                        ));
                    } else {
                        ctx.diagnostic(Self::restricted_test_method_with_message(
                            method_name,
                            &message,
                            span,
                        ));
                    }
                },
            );
        }
    }

    #[inline]
    fn compile_restricted_jest_methods(
        matchers: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<FxHashMap<String, String>> {
        Some(
            matchers
                .iter()
                .map(|(key, value)| {
                    (String::from(key), String::from(value.as_str().unwrap_or_default()))
                })
                .collect(),
        )
    }

    #[inline]
    fn restricted_test_method(method_name: &str, x0: &str, span1: Span) -> OxcDiagnostic {
        OxcDiagnostic::warn(format!("Disallow specific `{method_name}.` methods"))
            .with_help(format!("Use of `{x0:?}` is disallowed"))
            .with_label(span1)
    }

    #[inline]
    fn restricted_test_method_with_message(
        method_name: &str,
        x0: &str,
        span1: Span,
    ) -> OxcDiagnostic {
        OxcDiagnostic::warn(format!("Disallow specific `{method_name}.` methods"))
            .with_help(format!("{x0:?}"))
            .with_label(span1)
    }
}
