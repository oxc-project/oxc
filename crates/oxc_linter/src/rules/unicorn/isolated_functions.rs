use oxc_ast::{
    AstKind,
    ast::{
        Argument, CallExpression, ExportDefaultDeclarationKind, Expression, Function, FunctionBody,
        IdentifierReference, TSTypeReference,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{IsGlobalReference, ReferenceId, Scoping};
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{AstNode, config::GlobalValue, context::LintContext, rule::Rule};
use oxc_ast_visit::Visit;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsolatedFunctionOptions {
    ///Array of function names that create isolated execution contexts.
    ///Functions passed as arguments to these functions will be considered isolated.
    #[serde(default = "default_functions")]
    pub functions: Vec<String>,
    ///Array of ESLint selectors to identify isolated functions.
    ///  Useful for custom naming conventions or framework-specific patterns.
    #[serde(default)]
    pub selectors: Vec<String>,
    ///Array of comment strings that mark functions as isolated.
    /// Functions with inline, block, or JSDoc comments tagged with these strings will be considered isolated.
    /// (Definition of "tagged": either the comment consists solely of the tag, or starts with it,
    /// and has an explanation following a hyphen,
    ///  like // @isolated - this function will be stringified).
    #[serde(default = "default_comments")]
    pub comments: Vec<String>,
    ///  Controls how global variables are handled. When not specified, uses ESLint's language options globals.
    /// When specified as an object, each key is a global variable name and the value controls its behavior:
    /// 'readonly': Global variable is allowed but cannot be written to
    /// 'writable': Global variable is allowed and can be read/written
    /// 'off': Global variable is not allowed
    pub override_globals: Option<FxHashMap<String, GlobalValue>>,
}

fn default_functions() -> Vec<String> {
    vec!["makeSynchronous".into()]
}

fn default_comments() -> Vec<String> {
    vec!["@isolated".into()]
}

impl Default for IsolatedFunctionOptions {
    fn default() -> Self {
        Self {
            functions: default_functions(),
            selectors: Vec::new(),
            comments: default_comments(),
            override_globals: None,
        }
    }
}

impl TryFrom<Value> for IsolatedFunctionOptions {
    type Error = OxcDiagnostic;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let Some(config) = value.get(0) else { return Ok(Self::default()) };
        match config {
            Value::Object(config) => {
                let functions = config
                    .get("functions") // Fixed typo in field name
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter().filter_map(|item| item.as_str().map(|s| s.to_string())).collect()
                    })
                    .unwrap_or(default_functions());

                let selectors: Vec<String> = config
                    .get("selectors")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter().filter_map(|item| item.as_str().map(|s| s.to_string())).collect()
                    })
                    .unwrap_or_default();

                let comments: Vec<String> = config
                    .get("comments")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter().filter_map(|item| item.as_str().map(|s| s.to_string())).collect()
                    })
                    .unwrap_or(default_comments());

                let override_globals: Option<FxHashMap<String, GlobalValue>> =
                    config.get("overrideGlobals").and_then(|v| v.as_object()).map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| {
                                let s = match v {
                                    Value::String(str) => GlobalValue::try_from(str.as_str()),
                                    Value::Bool(bool) => Ok(GlobalValue::from(*bool)),
                                    _ => Err("Invalid global value"),
                                };

                                match s {
                                    Ok(value) => Some((k.clone(), value)),
                                    Err(_) => None,
                                }
                            })
                            .collect()
                    });

                Ok(Self { functions, selectors, comments, override_globals })
            }
            Value::Null => Ok(Self::default()),
            _ => Err(OxcDiagnostic::error(format!(
                "Invalid configuration for isolated-functions rule: Expected an object, got {config}"
            ))),
        }
    }
}

fn isolated_functions_diagnostic(message: String, help_text: String, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message).with_help(help_text).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct IsolatedFunctions(Box<IsolatedFunctionOptions>);

// See <https://github.com/sindresorhus/eslint-plugin-unicorn/blob/main/docs/rules/isolated-function.md> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule ensures that functions which are meant to be isolated from their surrounding scope are checked for external variable usage.
    /// It identifies functions that need to be isolated (via function names, JSDoc annotations, or selectors) and warns when they reference variables from the outer scope.
    ///
    /// ### Why is this bad?
    ///
    /// Functions that are isolated (executed in workers, serialized, etc.) cannot access variables from their surrounding scope.
    /// Using external variables in such functions will lead to runtime errors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const url = 'https://example.com';
    /// const getText = makeSynchronous(async () => {
    ///     const response = await fetch(url); // 'url' is not defined in isolated function scope
    ///     return response.text();
    /// });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const getText = makeSynchronous(async (url) => { // Variable passed as parameter
    ///     const response = await fetch(url);
    ///     return response.text();
    /// });
    /// ```
    IsolatedFunctions,
    unicorn,
    correctness,
    pending,
    config = IsolatedFunctionOptions
);

impl Rule for IsolatedFunctions {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        Ok(Self(Box::new(IsolatedFunctionOptions::try_from(value).unwrap_or_default())))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call) => {
                let is_matched = self.is_matched_functions(ctx, call);
                if is_matched {
                    self.check_function_scope(ctx, call);
                }
            }
            AstKind::Function(func) => {
                if self.get_isolated_comment(ctx, func) {
                    let function_body = func.body.as_ref().unwrap();
                    self.detect_function_body(ctx, function_body);
                }
            }
            AstKind::VariableDeclaration(decl) => {
                let declarator = &decl.declarations;
                if declarator.len() == 1
                    && matches!(declarator[0].init, Some(Expression::ArrowFunctionExpression(_)))
                    && let Some(Expression::ArrowFunctionExpression(arrow_func)) =
                        &declarator[0].init
                {
                    let function_body = &arrow_func.body;
                    self.detect_function_body(ctx, function_body);
                }
            }
            AstKind::ExportDefaultDeclaration(export_decl) => {
                let declarator = &export_decl.declaration;
                if matches!(declarator, ExportDefaultDeclarationKind::ArrowFunctionExpression(_))
                    && let ExportDefaultDeclarationKind::ArrowFunctionExpression(arrow_func) =
                        declarator
                {
                    let function_body = &arrow_func.body;

                    self.detect_function_body(ctx, function_body);
                }
            }

            _ => {}
        }
    }
}

impl IsolatedFunctions {
    fn detect_function_body<'a>(&self, ctx: &LintContext<'a>, function_body: &FunctionBody<'a>) {
        let scoping = ctx.scoping();
        let mut reference_collector = ReferenceCollector::new(scoping);

        reference_collector.visit_function_body(function_body);
        self.detect_symbols_is_in_scope(
            ctx,
            &reference_collector.references,
            &reference_collector.refereces_to_node,
        );
    }

    fn is_matched_functions<'a>(&self, _ctx: &LintContext<'a>, node: &CallExpression<'a>) -> bool {
        let callee = &node.callee;
        if let Expression::Identifier(ident) = callee
            && self.0.functions.contains(&ident.name.into())
        {
            return true;
        }
        false
    }

    fn check_function_scope<'a>(&self, ctx: &LintContext<'a>, node: &CallExpression<'a>) {
        let argument = &node.arguments;
        if argument.len() == 1 {
            let argument: &oxc_ast::ast::Argument<'_> = &argument[0];

            match argument {
                Argument::ArrowFunctionExpression(arrow_func) => {
                    let function_body = &arrow_func.body;
                    self.detect_function_body(ctx, function_body);
                }
                Argument::FunctionExpression(function) => {
                    if let Some(function_body) = &function.body {
                        self.detect_function_body(ctx, function_body);
                    }
                }

                _ => {}
            }
        }
    }

    fn detect_symbols_is_in_scope<'a>(
        &self,
        ctx: &LintContext<'a>,
        references: &[ReferenceId],
        references_to_node: &FxHashMap<ReferenceId, ReferenceVariableDesc>,
    ) {
        let mut collect_nodes: FxHashMap<ReferenceId, (String, Span)> = FxHashMap::default();
        let scoping = ctx.scoping();

        for reference_id in references {
            let reference_desc =
                references_to_node.get(reference_id).expect("cannot find a reference");

            if reference_desc.is_global {
                self.detect_global_reference(ctx, &reference_desc);
            } else {
                let reference = scoping.get_reference(*reference_id);
                let reference_scope_id = reference.scope_id();
                let symbol_id = &reference.symbol_id().expect("reference cannot find symbol");
                let symbol_decl_scope_id = scoping.symbol_scope_id(*symbol_id);
                let reference_variable_desc = references_to_node
                    .get(&reference_id)
                    .expect("cannot find valid reference node");

                if symbol_decl_scope_id != reference_scope_id && symbol_decl_scope_id == 0 {
                    if !collect_nodes.contains_key(reference_id) {
                        collect_nodes.insert(
                            *reference_id,
                            (reference_variable_desc.name.clone(), reference_variable_desc.span),
                        );
                    }
                }
            }

            if !collect_nodes.is_empty() {
                for (_, (name, span)) in &collect_nodes {
                    let message = format!("'{name}' is not defined in isolated function scope");
                    let helper_text = "Move all necessary variables inside the function or pass them as parameters.".into();
                    ctx.diagnostic(isolated_functions_diagnostic(message, helper_text, *span));
                }
            }
        }
    }

    fn detect_global_reference<'a>(
        &self,
        ctx: &LintContext<'a>,
        reference_variable_desc: &ReferenceVariableDesc,
    ) {
        match self.0.override_globals.as_ref() {
            Some(override_globals) => {
                let is_allowed =
                    self.is_allowed_global_variable(reference_variable_desc, override_globals);
                if !is_allowed {
                    let name = reference_variable_desc.name.clone();
                    let span = &reference_variable_desc.span;
                    let message = format!(
                        "'{name}' is a global variable, and its usage conflicts with the overrideGlobals option."
                    );
                    let helper_text = "fix overrideGlobals in config file.".into();
                    ctx.diagnostic(isolated_functions_diagnostic(message, helper_text, *span));
                }
            }
            None => {
                if matches!(reference_variable_desc.flag, ReferenceFlag::Write) {
                    let name = reference_variable_desc.name.clone();
                    let message = format!(
                        " overrideGlobals: '{name}' is a global variable, and its usage conflicts with the default property."
                    );
                    ctx.diagnostic(isolated_functions_diagnostic(
                        message,
                        "add overrideGlobals option in config file.".into(),
                        reference_variable_desc.span,
                    ));
                }
            }
        }
    }

    fn is_allowed_global_variable(
        &self,
        reference_desc: &ReferenceVariableDesc,
        override_globals: &FxHashMap<String, GlobalValue>,
    ) -> bool {
        let allowed_value = override_globals.get(&reference_desc.name);
        match allowed_value {
            Some(GlobalValue::Off) => return false,
            Some(GlobalValue::Readonly) => {
                if matches!(reference_desc.flag, ReferenceFlag::Read) {
                    return true;
                } else {
                    return false;
                }
            }
            Some(GlobalValue::Writable) => true,

            None => true,
        }
    }

    fn get_isolated_comment<'a>(&self, ctx: &LintContext<'a>, node: &Function) -> bool {
        let source_text = ctx.source_text();
        let comment_options = &self.0.comments;
        let comments = ctx.semantic().comments();
        for comment in comments {
            if comment.attached_to == node.span.start {
                let comment_text = comment.content_span().source_text(source_text);

                // find JSDoc comment cotains comments options /** @isolated */
                if comment.is_jsdoc() && Self::is_allowed_comment(comment_options, comment_text) {
                    return true;
                }

                // find line comment cotains comments options //@isolated
                if comment.is_line() && Self::is_allowed_comment(comment_options, comment_text) {
                    return true;
                }

                // find block comment cotains comments options /* @isolated */
                if comment.is_block() && Self::is_allowed_comment(comment_options, comment_text) {
                    return true;
                }
            }
        }
        false
        // if comment
    }

    fn is_allowed_comment(options: &Vec<String>, comment_content: &str) -> bool {
        options.iter().any(|option| comment_content.contains(option))
    }
}
#[derive(Debug)]
struct ReferenceVariableDesc {
    name: String,
    is_global: bool,
    span: Span,
    flag: ReferenceFlag,
}

#[derive(Debug)]
enum ReferenceFlag {
    Read,
    Write,
}

#[derive(Debug)]
pub struct ReferenceCollector<'a> {
    scoping: &'a Scoping,
    references: Vec<ReferenceId>,
    refereces_to_node: FxHashMap<ReferenceId, ReferenceVariableDesc>,
}

impl<'a> ReferenceCollector<'a> {
    fn new(scoping: &'a oxc_semantic::Scoping) -> Self {
        Self { scoping, references: Vec::new(), refereces_to_node: FxHashMap::default() }
    }
}

impl<'a> Visit<'a> for ReferenceCollector<'a> {
    fn visit_ts_type_reference(&mut self, _node: &TSTypeReference<'a>) {
        return;
    }
    fn visit_identifier_reference(&mut self, node: &IdentifierReference<'a>) {
        let Some(reference_id) = node.reference_id.get() else {
            return;
        };

        let reference = self.scoping.get_reference(reference_id);
        let is_global_reference = node.is_global_reference(self.scoping);
        let is_read = reference.is_read();

        if is_global_reference {
            if !self.refereces_to_node.contains_key(&reference_id) {
                self.references.push(reference_id);
                self.refereces_to_node.insert(
                    reference_id,
                    ReferenceVariableDesc {
                        name: node.name.to_string(),
                        is_global: true,
                        span: node.span,
                        flag: if is_read { ReferenceFlag::Read } else { ReferenceFlag::Write },
                    },
                );
            }
        } else {
            if !self.refereces_to_node.contains_key(&reference_id) && is_read {
                self.references.push(reference_id);
                self.refereces_to_node.insert(
                    reference_id,
                    ReferenceVariableDesc {
                        name: node.name.to_string(),
                        is_global: false,
                        span: node.span,
                        flag: ReferenceFlag::Read,
                    },
                );
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "makeSynchronous(() => {
                const foo = 'hi';
                return foo.slice();
            });",
            None,
        ),
        (
            "makeSynchronous(foo => {
                return foo.slice();
            });",
            None,
        ),
        (
            "/** @isolated */
            function abc () {
                const foo = 'hi';
                const slice = () => foo.slice();
                return slice();
            }",
            None,
        ),
        (
            "makeSynchronous(async function (foo) {
                return foo.slice();
            });",
            None,
        ),
        (
            r#"makeSynchronous(() => process.env.MAP ? new Map() : new URL("https://example.com"))"#,
            None,
        ),
        ("makeSynchronous(() => new Array())", None),
        ("makeSynchronous(() => new Array())", Some(serde_json::json!([{"overrideGlobals": {}}]))),
        (
            "makeSynchronous(function () {
                return foo.slice();
            });",
            None,
        ), // {"globals": {"foo": true}},
        (
            "makeSynchronous(function () {
                return foo.slice();
            });",
            Some(serde_json::json!([{"overrideGlobals": {"foo": true}}])),
        ), // {"globals": {"abc": true}},
        (
            "const a = 1;
                     type MyType = { foo: string };
                     makeSynchronous(() => {
                         const b: typeof a = 1;
                         const f = <T extends MyType>(t: T) => t;
                         let myType: MyType = { foo: 'bar' };
                         myType = { foo: 'bar' } as MyType;
                         myType = { foo: 'bar' } as const;
                         myType = { foo: 'baz' } satisfies MyType;
                         type X = typeof myType extends MyType ? true : false;
                     });",
            None,
        ), // { "parser": parsers.typescript, }
    ];

    let fail = vec![
        (
            "const foo = 'hi';
            makeSynchronous(() => foo.slice());",
            None,
        ),
        (
            "const foo = 'hi';
            makeSynchronous(async () => foo.slice());",
            None,
        ),
        (
            "const foo = 'hi';
            makeSynchronous(function () {
                return foo.slice();
            });",
            None,
        ),
        (
            "const foo = 'hi';
            makeSynchronous(async function () {
                return foo.slice();
            });",
            None,
        ),
        (
            "const foo = 'hi';
            makeSynchronous(function abc () {
                return foo.slice();
            });",
            None,
        ),
        (
            "const foo = 'hi';
            makeSynchronous(async function abc () {
                return foo.slice();
            });",
            None,
        ),
        (
            "const foo = 'hi';
            /** @isolated */
            function abc () {
                return foo.slice();
            }",
            None,
        ),
        (
            "const foo = 'hi';
            /** @isolated */
            const abc = () => foo.slice();",
            None,
        ),
        (
            "const foo = 'hi';
            // @isolated - explanation
            const abc1 = () => foo.slice();
            // @isolated -- explanation
            const abc2 = () => foo.slice();",
            None,
        ),
        (
            "const foo = 'hi';
            /* @isolated */
            const abc1 = () => foo.slice();
            /** @isolated */
            const abc2 = () => foo.slice();
            /**
             * @isolated
             */
            const abc3 = () => foo.slice();",
            None,
        ),
        (
            "const foo = 'hi';
            // @isolated
            const abc = () => foo.slice();",
            None,
        ),
        (
            "const foo = 'hi';
            // @isolated
            export const abc = () => foo.slice();
            // @isolated
            export default () => foo.slice();",
            None,
        ),
        (
            "makeSynchronous(function () {
                return new URL('https://example.com?') + new URLSearchParams({a: 'b'}).toString();
            });",
            Some(
                serde_json::json!([{"overrideGlobals": {"URLSearchParams": "readonly", "URL": "off"}}]),
            ),
        ),
        (
            "makeSynchronous(function () {
                location = new URL('https://example.com');
                process = {env: {}};
                process.env.FOO = 'bar';
            });",
            None,
        ),
        // (
        //     "const foo = 'hi';
        //     function lambdaHandlerFoo() {
        //         return foo.slice();
        //     }
        //     function someOtherFunction() {
        //         return foo.slice();
        //     }
        //     createLambda({
        //         name: 'fooLambda',
        //         code: lambdaHandlerFoo.toString(),
        //     });",
        //     Some(
        //         serde_json::json!([{"selectors": ["FunctionDeclaration[id.name=/lambdaHandler.*/]"]}]),
        //     ),
        // ),
        (
            "makeSynchronous(() => new Array())",
            Some(serde_json::json!([{"overrideGlobals": {"Array": "off"}}])),
        ),
    ];

    Tester::new(IsolatedFunctions::NAME, IsolatedFunctions::PLUGIN, pass, fail).test_and_snapshot();
}
