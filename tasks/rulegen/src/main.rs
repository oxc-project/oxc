#![allow(clippy::print_stdout, clippy::print_stderr, clippy::disallowed_methods)]
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use convert_case::{Case, Casing};
use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, CallExpression, ExportDefaultDeclarationKind, Expression,
        ExpressionStatement, ObjectExpression, ObjectProperty, ObjectPropertyKind, Program,
        PropertyKey, Statement, StaticMemberExpression, StringLiteral, TaggedTemplateExpression,
        TemplateLiteral,
    },
    Visit,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use rustc_hash::FxHashMap;
use serde::Serialize;
use ureq::Response;

mod json;
mod template;

const ESLINT_TEST_PATH: &str =
    "https://raw.githubusercontent.com/eslint/eslint/main/tests/lib/rules";

const JEST_TEST_PATH: &str =
    "https://raw.githubusercontent.com/jest-community/eslint-plugin-jest/main/src/rules/__tests__";

const TYPESCRIPT_ESLINT_TEST_PATH: &str = "https://raw.githubusercontent.com/typescript-eslint/typescript-eslint/main/packages/eslint-plugin/tests/rules";

const UNICORN_TEST_PATH: &str =
    "https://raw.githubusercontent.com/sindresorhus/eslint-plugin-unicorn/main/test";

const IMPORT_TEST_PATH: &str =
    "https://raw.githubusercontent.com/import-js/eslint-plugin-import/main/tests/src/rules";

const REACT_TEST_PATH: &str =
    "https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-react/master/tests/lib/rules";

const JSX_A11Y_TEST_PATH: &str =
    "https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-jsx-a11y/main/__tests__/src/rules";

const NEXT_JS_TEST_PATH: &str =
    "https://raw.githubusercontent.com/vercel/next.js/canary/test/unit/eslint-plugin-next";

const JSDOC_TEST_PATH: &str =
    "https://raw.githubusercontent.com/gajus/eslint-plugin-jsdoc/main/test/rules/assertions";

const REACT_PERF_TEST_PATH: &str =
    "https://raw.githubusercontent.com/cvazac/eslint-plugin-react-perf/main/tests/lib/rules";

const NODE_TEST_PATH: &str =
    "https://raw.githubusercontent.com/eslint-community/eslint-plugin-n/master/tests/lib/rules";

const TREE_SHAKING_PATH: &str =
    "https://raw.githubusercontent.com/lukastaegert/eslint-plugin-tree-shaking/master/src/rules";

const PROMISE_TEST_PATH: &str =
    "https://raw.githubusercontent.com/eslint-community/eslint-plugin-promise/main/__tests__";

const VITEST_TEST_PATH: &str =
    "https://raw.githubusercontent.com/veritem/eslint-plugin-vitest/main/tests";

struct TestCase {
    source_text: String,
    code: Option<String>,
    output: Option<String>,
    group_comment: Option<String>,
    config: Option<String>,
    settings: Option<String>,
    filename: Option<String>,
    language_options: Option<String>,
}

impl TestCase {
    fn new(source_text: &str, arg: &Expression<'_>) -> Self {
        let mut test_case = Self {
            source_text: source_text.to_string(),
            code: None,
            output: None,
            config: None,
            settings: None,
            group_comment: None,
            filename: None,
            language_options: None,
        };
        test_case.visit_expression(arg);
        test_case
    }

    fn with_group_comment(mut self, comment: String) -> Self {
        self.group_comment = Some(comment);
        self
    }

    fn code(&self, need_config: bool, need_settings: bool, need_filename: bool) -> String {
        self.code
            .as_ref()
            .map(|code| {
                let code_str = format_code_snippet(code);
                let config = self.config.as_ref().map_or_else(
                    || "None".to_string(),
                    |config| format!("Some(serde_json::json!({config}))"),
                );
                let settings = self.settings.as_ref().map_or_else(
                    || "None".to_string(),
                    |settings| format!(r#"Some(serde_json::json!({{ "settings": {settings} }}))"#),
                );
                let filename = self.filename.as_ref().map_or_else(
                    || "None".to_string(),
                    |filename| format!(r#"Some(PathBuf::from("{filename}"))"#),
                );
                let code_str = if need_filename {
                    format!("({code_str}, {config}, {settings}, {filename})")
                } else if need_settings {
                    format!("({code_str}, {config}, {settings})")
                } else if need_config {
                    format!("({code_str}, {config})")
                } else {
                    code_str
                };
                if let Some(language_options) = &self.language_options {
                    format!("{code_str}, // {language_options}")
                } else {
                    code_str
                }
            })
            .unwrap_or_default()
    }

    fn group_comment(&self) -> Option<&str> {
        self.group_comment.as_deref()
    }

    fn output(&self) -> Option<String> {
        let code = format_code_snippet(self.code.as_ref()?);
        let output = format_code_snippet(self.output.as_ref()?);
        let config = self.config.as_ref().map_or_else(
            || "None".to_string(),
            |config| format!("Some(serde_json::json!({config}))"),
        );

        // ("null==null", "null === null", None),
        Some(format!(r#"({code}, {output}, {config})"#))
    }
}

fn format_code_snippet(code: &str) -> String {
    let code = if code.contains('\n') {
        code.replace('\n', "\n\t\t\t").replace('\\', "\\\\").replace('\"', "\\\"")
    } else {
        code.to_string()
    };

    // Do not quote strings that are already raw strings
    if code.starts_with("r\"") || code.starts_with("r#\"") {
        return code;
    }

    // "debugger" => "debugger"
    if !code.contains('"') {
        return format!("\"{code}\"");
    }

    // "document.querySelector("#foo");" => r##"document.querySelector("#foo");"##
    if code.contains("\"#") {
        return format!("r##\"{code}\"##");
    }

    // 'import foo from "foo";' => r#"import foo from "foo";"#
    format!("r#\"{}\"#", code.replace("\\\"", "\""))
}

impl<'a> Visit<'a> for TestCase {
    fn visit_expression(&mut self, expr: &Expression<'a>) {
        match expr {
            Expression::StringLiteral(lit) => self.visit_string_literal(lit),
            Expression::TemplateLiteral(lit) => self.visit_template_literal(lit),
            Expression::ObjectExpression(obj_expr) => self.visit_object_expression(obj_expr),
            Expression::CallExpression(call_expr) => self.visit_call_expression(call_expr),
            Expression::TaggedTemplateExpression(tag_expr) => {
                self.visit_tagged_template_expression(tag_expr);
            }
            _ => {}
        }
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Some(member_expr) = expr.callee.as_member_expression() {
            if let Expression::ArrayExpression(array_expr) = member_expr.object() {
                // ['class A {', '}'].join('\n')
                let mut code = String::new();
                for arg in &array_expr.elements {
                    let ArrayExpressionElement::StringLiteral(lit) = arg else {
                        continue;
                    };
                    code.push_str(lit.value.as_str());
                    code.push('\n');
                }
                self.code = Some(code);
                self.config = None;
            }
        }
    }

    fn visit_object_expression(&mut self, expr: &ObjectExpression<'a>) {
        for obj_prop in &expr.properties {
            match obj_prop {
                ObjectPropertyKind::ObjectProperty(prop) => match &prop.key {
                    PropertyKey::StaticIdentifier(ident) if ident.name == "code" => {
                        self.code = match &prop.value {
                            Expression::StringLiteral(s) => Some(s.value.to_string()),
                            Expression::TaggedTemplateExpression(tag_expr) => {
                                // If it is a raw string like String.raw`something`, then we import that as a Rust raw string literal
                                if tag_expr.tag.is_specific_member_access("String", "raw") {
                                    tag_expr
                                        .quasi
                                        .quasis
                                        .first()
                                        .map(|quasi| format!("r#\"{}\"#", quasi.value.raw))
                                } else {
                                    // There are `dedent`(in eslint-plugin-jest), `outdent`(in eslint-plugin-unicorn) and `noFormat`(in typescript-eslint)
                                    // are known to be used to format test cases for their own purposes.
                                    // We read the quasi of tagged template directly also for the future usage.
                                    tag_expr.quasi.quasi().map(|quasi| quasi.to_string())
                                }
                            }
                            Expression::TemplateLiteral(tag_expr) => {
                                tag_expr.quasi().map(|quasi| quasi.to_string())
                            }
                            // handle code like ["{", "a: 1", "}"].join("\n")
                            Expression::CallExpression(call_expr) => {
                                if !call_expr.arguments.first().is_some_and(|arg|  matches!(arg, Argument::StringLiteral(string) if string.value == "\n")) {
                                    continue;
                                }
                                let Expression::StaticMemberExpression(member) = &call_expr.callee
                                else {
                                    continue;
                                };
                                if member.property.name != "join" {
                                    continue;
                                }
                                let Expression::ArrayExpression(array_expr) = &member.object else {
                                    continue;
                                };
                                Some(
                                    array_expr
                                        .elements
                                        .iter()
                                        .map(|arg| match arg {
                                            ArrayExpressionElement::StringLiteral(string) => {
                                                string.value.as_str()
                                            }
                                            _ => "",
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                )
                            }
                            _ => continue,
                        }
                    }
                    PropertyKey::StaticIdentifier(ident) if ident.name == "output" => {
                        self.output = match &prop.value {
                            Expression::StringLiteral(s) => Some(s.value.to_string()),
                            Expression::TaggedTemplateExpression(tag_expr) => {
                                tag_expr.quasi.quasi().map(|quasi| quasi.to_string())
                            }
                            Expression::TemplateLiteral(tag_expr) => {
                                tag_expr.quasi().map(|quasi| quasi.to_string())
                            }
                            _ => None,
                        }
                    }
                    PropertyKey::StaticIdentifier(ident) if ident.name == "options" => {
                        let span = prop.value.span();
                        let option_text = &self.source_text[span.start as usize..span.end as usize];
                        self.config = Some(json::convert_config_to_json_literal(option_text));
                    }
                    PropertyKey::StaticIdentifier(ident) if ident.name == "settings" => {
                        let span = prop.value.span();
                        let setting_text = span.source_text(&self.source_text);
                        self.settings = Some(json::convert_config_to_json_literal(setting_text));
                    }
                    PropertyKey::StaticIdentifier(ident) if ident.name == "filename" => {
                        let span = prop.value.span();
                        let filename = span.source_text(&self.source_text);
                        self.filename = Some(filename.to_string());
                    }
                    PropertyKey::StaticIdentifier(ident) if ident.name == "languageOptions" => {
                        let span = prop.value.span();
                        let language_options = span.source_text(&self.source_text);
                        let language_options =
                            json::convert_config_to_json_literal(language_options);
                        self.language_options = Some(language_options);
                    }
                    _ => continue,
                },
                ObjectPropertyKind::SpreadProperty(_) => continue,
            }
        }
    }

    fn visit_template_literal(&mut self, lit: &TemplateLiteral<'a>) {
        self.code = Some(lit.quasi().unwrap().to_string());
        self.config = None;
    }

    fn visit_string_literal(&mut self, lit: &StringLiteral) {
        self.code = Some(lit.value.to_string());
        self.config = None;
    }

    fn visit_tagged_template_expression(&mut self, expr: &TaggedTemplateExpression<'a>) {
        if expr.tag.is_specific_id("dedent") || expr.tag.is_specific_id("outdent") {
            return;
        }

        // If it is a raw string like String.raw`something`, then we import that as a Rust raw string literal
        self.code = if expr.tag.is_specific_member_access("String", "raw") {
            expr.quasi.quasis.first().map(|quasi| format!("r#\"{}\"#", quasi.value.raw))
        } else {
            expr.quasi.quasi().map(|quasi| quasi.to_string())
        };
        self.config = None;
    }
}

#[derive(Serialize)]
pub struct Context {
    plugin_name: String,
    kebab_rule_name: String,
    pascal_rule_name: String,
    snake_rule_name: String,
    pass_cases: String,
    fail_cases: String,
    fix_cases: Option<String>,
    has_filename: bool,
    /// Language examples are written in.
    ///
    /// Should be `"js"`, `"jsx"`, `"ts"`, `"tsx"`. Defaults to `"js"`.
    language: Cow<'static, str>,
}

impl Context {
    fn new(plugin_name: String, rule_name: &str, pass_cases: String, fail_cases: String) -> Self {
        let pascal_rule_name = rule_name.to_case(Case::Pascal);
        let kebab_rule_name = rule_name.to_case(Case::Kebab);
        let underscore_rule_name = rule_name.to_case(Case::Snake);
        Self {
            plugin_name,
            kebab_rule_name,
            pascal_rule_name,
            snake_rule_name: underscore_rule_name,
            pass_cases,
            fail_cases,
            fix_cases: None,
            has_filename: false,
            language: Cow::Borrowed("js"),
        }
    }

    fn with_filename(mut self, has_filename: bool) -> Self {
        self.has_filename = has_filename;
        self
    }

    fn with_fix_cases(mut self, fix_cases: String) -> Self {
        self.fix_cases = Some(fix_cases);
        self
    }

    fn with_language<S: Into<Cow<'static, str>>>(mut self, language: S) -> Self {
        self.language = language.into();
        self
    }
}

struct State<'a> {
    source_text: &'a str,
    valid_tests: Vec<&'a Expression<'a>>,
    invalid_tests: Vec<&'a Expression<'a>>,
    expression_to_group_comment_map: FxHashMap<Span, String>,
    group_comment_stack: Vec<String>,
}

impl<'a> State<'a> {
    fn new(source_text: &'a str) -> Self {
        Self {
            source_text,
            valid_tests: vec![],
            invalid_tests: vec![],
            expression_to_group_comment_map: FxHashMap::default(),
            group_comment_stack: vec![],
        }
    }

    fn pass_cases(&self) -> Vec<TestCase> {
        self.get_test_cases(&self.valid_tests)
    }

    fn fail_cases(&self) -> Vec<TestCase> {
        self.get_test_cases(&self.invalid_tests)
    }

    fn get_test_cases(&self, tests: &[&'a Expression<'a>]) -> Vec<TestCase> {
        tests
            .iter()
            .map(|arg| {
                let case = TestCase::new(self.source_text, arg);
                if let Some(group_comment) = self.expression_to_group_comment_map.get(&arg.span()) {
                    case.with_group_comment(group_comment.to_string())
                } else {
                    case
                }
            })
            .collect::<Vec<_>>()
    }

    fn get_comment(&self) -> String {
        self.group_comment_stack.join(" ")
    }

    fn add_valid_test(&mut self, expr: &'a Expression<'a>) {
        self.valid_tests.push(expr);
        self.expression_to_group_comment_map.insert(expr.span(), self.get_comment());
    }

    fn add_invalid_test(&mut self, expr: &'a Expression<'a>) {
        self.invalid_tests.push(expr);
        self.expression_to_group_comment_map.insert(expr.span(), self.get_comment());
    }
}

impl<'a> Visit<'a> for State<'a> {
    fn visit_program(&mut self, program: &Program<'a>) {
        for stmt in &program.body {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &Statement<'a>) {
        match stmt {
            Statement::ExpressionStatement(expr_stmt) => self.visit_expression_statement(expr_stmt),
            // for eslint-plugin-jsdoc
            Statement::ExportDefaultDeclaration(export_decl) => {
                if let ExportDefaultDeclarationKind::ObjectExpression(obj_expr) =
                    &export_decl.declaration
                {
                    self.visit_object_expression(obj_expr);
                }
            }
            _ => {}
        }
    }

    fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
        self.visit_expression(&stmt.expression);
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        let mut pushed = false;
        if let Expression::Identifier(ident) = &expr.callee {
            // Add describe's first parameter as part group comment
            // e.g. for `describe('valid', () => { ... })`, the group comment will be "valid"
            if ident.name == "describe" {
                if let Some(Argument::StringLiteral(lit)) = expr.arguments.first() {
                    pushed = true;
                    self.group_comment_stack.push(lit.value.to_string());
                }
            }
        }
        for arg in &expr.arguments {
            self.visit_argument(arg);
        }

        if pushed {
            self.group_comment_stack.pop();
        }

        self.visit_expression(&expr.callee);
    }

    fn visit_object_property(&mut self, prop: &ObjectProperty<'a>) {
        let PropertyKey::StaticIdentifier(ident) = &prop.key else { return };
        match ident.name.as_str() {
            "valid" => {
                if let Expression::ArrayExpression(array_expr) = &prop.value {
                    let array_expr = self.alloc(array_expr);
                    for arg in &array_expr.elements {
                        if let Some(expr) = arg.as_expression() {
                            self.add_valid_test(expr);
                        }
                    }
                }

                // for eslint-plugin-jsx-a11y
                if let Some(args) = find_parser_arguments(&prop.value).map(|args| self.alloc(args))
                {
                    for arg in args {
                        if let Some(expr) = arg.as_expression() {
                            self.add_valid_test(expr);
                        }
                    }
                }

                if let Expression::CallExpression(call_expr) = &prop.value {
                    if call_expr.callee.is_member_expression() {
                        // for eslint-plugin-react
                        if let Some(Argument::ArrayExpression(array_expr)) =
                            call_expr.arguments.first()
                        {
                            let array_expr = self.alloc(array_expr);
                            for arg in &array_expr.elements {
                                if let Some(expr) = arg.as_expression() {
                                    self.add_valid_test(expr);
                                }
                            }
                        }
                    }
                }
            }
            "invalid" => {
                if let Expression::ArrayExpression(array_expr) = &prop.value {
                    let array_expr = self.alloc(array_expr);
                    for arg in &array_expr.elements {
                        if let Some(expr) = arg.as_expression() {
                            self.add_invalid_test(expr);
                        }
                    }
                }

                // for eslint-plugin-jsx-a11y
                if let Some(args) = find_parser_arguments(&prop.value).map(|args| self.alloc(args))
                {
                    for arg in args {
                        if let Some(expr) = arg.as_expression() {
                            self.add_invalid_test(expr);
                        }
                    }
                }

                // for eslint-plugin-react
                if let Expression::CallExpression(call_expr) = &prop.value {
                    if call_expr.callee.is_member_expression() {
                        if let Some(Argument::ArrayExpression(array_expr)) =
                            call_expr.arguments.first()
                        {
                            let array_expr = self.alloc(array_expr);
                            for arg in &array_expr.elements {
                                if let Some(expr) = arg.as_expression() {
                                    self.add_invalid_test(expr);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn find_parser_arguments<'a, 'b>(
    mut expr: &'b Expression<'a>,
) -> Option<&'b oxc_allocator::Vec<'a, Argument<'a>>> {
    loop {
        let Expression::CallExpression(call_expr) = expr else { return None };
        let Expression::StaticMemberExpression(static_member_expr) = &call_expr.callee else {
            return None;
        };
        let StaticMemberExpression { object, property, .. } = &**static_member_expr;
        if let Expression::Identifier(iden) = object {
            if iden.name == "parsers" && property.name == "all" {
                if let Some(arg) = call_expr.arguments.first() {
                    if let Argument::CallExpression(call_expr) = arg {
                        if call_expr.callee.is_member_expression() {
                            return Some(&call_expr.arguments);
                        }
                        return None;
                    }
                    if arg.is_expression() {
                        return None;
                    }
                }
            }
        }
        expr = object;
    }
}

#[derive(Clone, Copy)]
pub enum RuleKind {
    ESLint,
    Jest,
    Typescript,
    Unicorn,
    Import,
    React,
    ReactPerf,
    JSXA11y,
    Oxc,
    NextJS,
    JSDoc,
    Node,
    TreeShaking,
    Promise,
    Vitest,
    Security,
}

impl RuleKind {
    fn from(kind: &str) -> Self {
        match kind {
            "jest" => Self::Jest,
            "typescript" => Self::Typescript,
            "unicorn" => Self::Unicorn,
            "import" => Self::Import,
            "react" => Self::React,
            "react-perf" => Self::ReactPerf,
            "jsx-a11y" => Self::JSXA11y,
            "oxc" => Self::Oxc,
            "nextjs" => Self::NextJS,
            "jsdoc" => Self::JSDoc,
            "n" => Self::Node,
            "tree-shaking" => Self::TreeShaking,
            "promise" => Self::Promise,
            "vitest" => Self::Vitest,
            "security" => Self::Security,
            _ => Self::ESLint,
        }
    }
}

impl Display for RuleKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ESLint => write!(f, "eslint"),
            Self::Typescript => write!(f, "typescript-eslint"),
            Self::Jest => write!(f, "eslint-plugin-jest"),
            Self::Unicorn => write!(f, "eslint-plugin-unicorn"),
            Self::Import => write!(f, "eslint-plugin-import"),
            Self::React => write!(f, "eslint-plugin-react"),
            Self::ReactPerf => write!(f, "eslint-plugin-react-perf"),
            Self::JSXA11y => write!(f, "eslint-plugin-jsx-a11y"),
            Self::Oxc => write!(f, "oxc"),
            Self::NextJS => write!(f, "eslint-plugin-next"),
            Self::JSDoc => write!(f, "eslint-plugin-jsdoc"),
            Self::Node => write!(f, "eslint-plugin-n"),
            Self::TreeShaking => write!(f, "eslint-plugin-tree-shaking"),
            Self::Promise => write!(f, "eslint-plugin-promise"),
            Self::Vitest => write!(f, "eslint-plugin-vitest"),
            Self::Security => write!(f, "security"),
        }
    }
}

fn main() {
    let mut args = std::env::args();
    args.next();

    let rule_name = args.next().expect("expected rule name").to_case(Case::Snake);
    let rule_kind = args.next().map_or(RuleKind::ESLint, |kind| RuleKind::from(&kind));
    let kebab_rule_name = rule_name.to_case(Case::Kebab);
    let camel_rule_name = rule_name.to_case(Case::Camel);
    let plugin_name = rule_kind.to_string();

    let rule_test_path = match rule_kind {
        RuleKind::ESLint => format!("{ESLINT_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::Jest => format!("{JEST_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::Typescript => format!("{TYPESCRIPT_ESLINT_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::Unicorn => format!("{UNICORN_TEST_PATH}/{kebab_rule_name}.mjs"),
        RuleKind::Import => format!("{IMPORT_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::React => format!("{REACT_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::ReactPerf => format!("{REACT_PERF_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::JSXA11y => format!("{JSX_A11Y_TEST_PATH}/{kebab_rule_name}-test.js"),
        RuleKind::NextJS => format!("{NEXT_JS_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::JSDoc => format!("{JSDOC_TEST_PATH}/{camel_rule_name}.js"),
        RuleKind::Node => format!("{NODE_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::TreeShaking => format!("{TREE_SHAKING_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::Promise => format!("{PROMISE_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::Vitest => format!("{VITEST_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::Oxc | RuleKind::Security => String::new(),
    };
    let language = match rule_kind {
        RuleKind::Typescript | RuleKind::Oxc => "ts",
        RuleKind::NextJS => "tsx",
        RuleKind::React | RuleKind::ReactPerf | RuleKind::JSXA11y | RuleKind::TreeShaking => "jsx",
        _ => "js",
    };

    println!("Reading test file from {rule_test_path}");

    let body = oxc_tasks_common::agent().get(&rule_test_path).call().map(Response::into_string);
    let context = match body {
        Ok(Ok(body)) => {
            let allocator = Allocator::default();
            let source_type = SourceType::from_path(rule_test_path).expect("incorrect {path:?}");
            let ret = Parser::new(&allocator, &body, source_type).parse();

            let mut state = State::new(&body);
            state.visit_program(&ret.program);

            let pass_cases = state.pass_cases();
            let fail_cases = state.fail_cases();
            println!(
                "File parsed and {} pass cases, {} fail cases are found",
                pass_cases.len(),
                fail_cases.len()
            );

            let pass_has_config = pass_cases.iter().any(|case| case.config.is_some());
            let fail_has_config = fail_cases.iter().any(|case| case.config.is_some());
            let has_config = pass_has_config || fail_has_config;

            let pass_has_settings = pass_cases.iter().any(|case| case.settings.is_some());
            let fail_has_settings = fail_cases.iter().any(|case| case.settings.is_some());
            let has_settings = pass_has_settings || fail_has_settings;

            let pass_has_filename = pass_cases.iter().any(|case| case.filename.is_some());
            let fail_has_filename = fail_cases.iter().any(|case| case.filename.is_some());
            let has_filename = pass_has_filename || fail_has_filename;

            let gen_cases_string = |cases: Vec<TestCase>| {
                let mut codes = vec![];
                let mut fix_codes = vec![];
                let mut last_comment = String::new();
                for case in cases {
                    let current_comment = case.group_comment();
                    let mut code = case.code(has_config, has_settings, has_filename);
                    if code.is_empty() {
                        continue;
                    }
                    if let Some(current_comment) = current_comment {
                        if current_comment != last_comment {
                            last_comment = current_comment.to_string();
                            code = format!(
                                "// {}\n{}",
                                &last_comment,
                                case.code(has_config, has_settings, has_filename)
                            );
                        }
                    }

                    if let Some(output) = case.output() {
                        fix_codes.push(output);
                    }

                    codes.push(code);
                }

                (codes.join(",\n"), fix_codes.join(",\n"))
            };

            // pass cases don't need to be fixed
            let (pass_cases, _) = gen_cases_string(pass_cases);
            let (fail_cases, fix_cases) = gen_cases_string(fail_cases);

            Context::new(plugin_name, &rule_name, pass_cases, fail_cases)
                .with_language(language)
                .with_filename(has_filename)
                .with_fix_cases(fix_cases)
        }
        Err(_err) => {
            println!("Rule {rule_name} cannot be found in {rule_kind}, use empty template.");
            Context::new(plugin_name, &rule_name, String::new(), String::new())
        }
        Ok(Err(err)) => {
            println!("Failed to convert rule source code to string: {err}, use empty template");
            Context::new(plugin_name, &rule_name, String::new(), String::new())
        }
    };

    let rule_name = &context.kebab_rule_name;
    let template = template::Template::with_context(&context);
    if let Err(err) = template.render(rule_kind) {
        eprintln!("failed to render {rule_name} rule template: {err}");
    }
}
