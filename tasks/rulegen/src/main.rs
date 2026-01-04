#![expect(clippy::print_stdout, clippy::print_stderr, clippy::disallowed_methods)]
use std::fmt::Write as _;
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use convert_case::{Case, Casing};
use lazy_regex::regex;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, AssignmentTarget, BindingPattern,
    CallExpression, Declaration, Expression, ExpressionStatement, IdentifierName, ObjectExpression,
    ObjectProperty, ObjectPropertyKind, Program, PropertyKey, Statement, StaticMemberExpression,
    StringLiteral, TaggedTemplateExpression, TemplateLiteral,
};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;

mod json;
mod template;
mod util;

const ESLINT_TEST_PATH: &str =
    "https://raw.githubusercontent.com/eslint/eslint/main/tests/lib/rules";
const ESLINT_RULES_PATH: &str = "https://raw.githubusercontent.com/eslint/eslint/main/lib/rules";

const JEST_TEST_PATH: &str =
    "https://raw.githubusercontent.com/jest-community/eslint-plugin-jest/main/src/rules/__tests__";
const JEST_RULES_PATH: &str =
    "https://raw.githubusercontent.com/jest-community/eslint-plugin-jest/main/src/rules";

const TYPESCRIPT_ESLINT_TEST_PATH: &str = "https://raw.githubusercontent.com/typescript-eslint/typescript-eslint/main/packages/eslint-plugin/tests/rules";
const TYPESCRIPT_ESLINT_RULES_PATH: &str = "https://raw.githubusercontent.com/typescript-eslint/typescript-eslint/main/packages/eslint-plugin/src/rules";

const UNICORN_TEST_PATH: &str =
    "https://raw.githubusercontent.com/sindresorhus/eslint-plugin-unicorn/main/test";
const UNICORN_RULES_PATH: &str =
    "https://raw.githubusercontent.com/sindresorhus/eslint-plugin-unicorn/main/rules";

const IMPORT_TEST_PATH: &str =
    "https://raw.githubusercontent.com/import-js/eslint-plugin-import/main/tests/src/rules";
const IMPORT_RULES_PATH: &str =
    "https://raw.githubusercontent.com/import-js/eslint-plugin-import/main/src/rules";

const REACT_TEST_PATH: &str =
    "https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-react/master/tests/lib/rules";
const REACT_RULES_PATH: &str =
    "https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-react/master/lib/rules";

const JSX_A11Y_TEST_PATH: &str =
    "https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-jsx-a11y/main/__tests__/src/rules";
const JSX_A11Y_RULES_PATH: &str =
    "https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-jsx-a11y/main/src/rules";

const NEXT_JS_TEST_PATH: &str =
    "https://raw.githubusercontent.com/vercel/next.js/canary/test/unit/eslint-plugin-next";
const NEXT_JS_RULES_PATH: &str =
    "https://raw.githubusercontent.com/vercel/next.js/canary/packages/eslint-plugin-next/src/rules";

const JSDOC_TEST_PATH: &str =
    "https://raw.githubusercontent.com/gajus/eslint-plugin-jsdoc/main/test/rules/assertions";
const JSDOC_RULES_PATH: &str =
    "https://raw.githubusercontent.com/gajus/eslint-plugin-jsdoc/main/src/rules";

const REACT_PERF_TEST_PATH: &str =
    "https://raw.githubusercontent.com/cvazac/eslint-plugin-react-perf/master/tests/lib/rules";
const REACT_PERF_RULES_PATH: &str =
    "https://raw.githubusercontent.com/cvazac/eslint-plugin-react-perf/master/lib/rules";

const NODE_TEST_PATH: &str =
    "https://raw.githubusercontent.com/eslint-community/eslint-plugin-n/master/tests/lib/rules";
const NODE_RULES_PATH: &str =
    "https://raw.githubusercontent.com/eslint-community/eslint-plugin-n/master/lib/rules";

const PROMISE_TEST_PATH: &str =
    "https://raw.githubusercontent.com/eslint-community/eslint-plugin-promise/main/__tests__";
const PROMISE_RULES_PATH: &str =
    "https://raw.githubusercontent.com/eslint-community/eslint-plugin-promise/main/rules";

const VITEST_TEST_PATH: &str =
    "https://raw.githubusercontent.com/vitest-dev/eslint-plugin-vitest/main/tests";
const VITEST_RULES_PATH: &str =
    "https://raw.githubusercontent.com/vitest-dev/eslint-plugin-vitest/main/src/rules";

const VUE_TEST_PATH: &str =
    "https://raw.githubusercontent.com/vuejs/eslint-plugin-vue/master/tests/lib/rules";
const VUE_RULES_PATH: &str =
    "https://raw.githubusercontent.com/vuejs/eslint-plugin-vue/master/lib/rules";

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
        Some(format!(r"({code}, {output}, {config})"))
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

    // `debugger` => `debugger`
    // `"debugger"` => `r#"\"debugger\""#`
    // `\u1234` => `r#"\u1234"`
    if !code.contains('"') && !code.contains('\\') {
        return format!("\"{code}\"");
    }

    // "document.querySelector("#foo");" => r##"document.querySelector("#foo");"##
    if code.contains("\"#") {
        return format!("r##\"{code}\"##");
    }

    // 'import foo from "foo";' => r#"import foo from "foo";"#
    format!("r#\"{}\"#", code.replace("\\\"", "\""))
}

// TODO: handle `noFormat`(in typescript-eslint)
fn format_tagged_template_expression(tag_expr: &TaggedTemplateExpression) -> Option<String> {
    if tag_expr.tag.is_specific_member_access("String", "raw") {
        tag_expr.quasi.quasis.first().map(|quasi| format!("r#\"{}\"#", quasi.value.raw))
    } else if tag_expr.tag.is_specific_id("dedent") || tag_expr.tag.is_specific_id("outdent") {
        tag_expr.quasi.quasis.first().map(|quasi| util::dedent(&quasi.value.raw))
    } else {
        tag_expr.quasi.single_quasi().map(|quasi| quasi.to_string())
    }
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
        if let Some(member_expr) = expr.callee.as_member_expression()
            && let Expression::ArrayExpression(array_expr) = member_expr.object()
        {
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

    fn visit_object_expression(&mut self, expr: &ObjectExpression<'a>) {
        for obj_prop in &expr.properties {
            match obj_prop {
                ObjectPropertyKind::ObjectProperty(prop) => match &prop.key {
                    PropertyKey::StaticIdentifier(ident) if ident.name == "code" => {
                        self.code = match &prop.value {
                            Expression::StringLiteral(s) => Some(s.value.to_string()),
                            Expression::TaggedTemplateExpression(tag_expr) => {
                                format_tagged_template_expression(tag_expr)
                            }
                            Expression::TemplateLiteral(tag_expr) => {
                                tag_expr.single_quasi().map(|quasi| quasi.to_string())
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
                                format_tagged_template_expression(tag_expr)
                            }
                            Expression::TemplateLiteral(tag_expr) => {
                                tag_expr.single_quasi().map(|quasi| quasi.to_string())
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
                        // trim quotes
                        let filename = filename.trim_matches('"').trim_matches('\'');
                        self.filename = Some(filename.to_string());
                    }
                    PropertyKey::StaticIdentifier(ident) if ident.name == "languageOptions" => {
                        let span = prop.value.span();
                        let language_options = span.source_text(&self.source_text);
                        let language_options =
                            json::convert_config_to_json_literal(language_options);
                        self.language_options = Some(language_options);
                    }
                    _ => {}
                },
                ObjectPropertyKind::SpreadProperty(_) => {}
            }
        }
    }

    fn visit_template_literal(&mut self, lit: &TemplateLiteral<'a>) {
        self.code = Some(
            lit.single_quasi()
                .expect("Expected template literal to have a single quasi")
                .to_string(),
        );
        self.config = None;
    }

    fn visit_string_literal(&mut self, lit: &StringLiteral) {
        self.code = Some(lit.value.to_string());
        self.config = None;
    }

    fn visit_tagged_template_expression(&mut self, expr: &TaggedTemplateExpression<'a>) {
        self.code = format_tagged_template_expression(expr);
        self.config = None;
    }
}

#[derive(Serialize)]
pub struct Context {
    mod_name: String,
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
    rule_config: Option<String>,
    rule_config_tuple: Option<String>,
    has_hash_map: bool,
    has_hash_set: bool,
}

impl Context {
    fn new(plugin_name: RuleKind, rule_name: &str, pass_cases: String, fail_cases: String) -> Self {
        let pascal_rule_name = rule_name.to_case(Case::Pascal);
        let kebab_rule_name = rule_name.to_case(Case::Kebab);
        let underscore_rule_name = rule_name.to_case(Case::Snake);
        let mod_name = get_mod_name(plugin_name);

        Self {
            mod_name,
            kebab_rule_name,
            pascal_rule_name,
            snake_rule_name: underscore_rule_name,
            pass_cases,
            fail_cases,
            fix_cases: None,
            has_filename: false,
            language: Cow::Borrowed("js"),
            rule_config: None,
            rule_config_tuple: None,
            has_hash_map: false,
            has_hash_set: false,
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

    fn with_rule_config(
        mut self,
        rule_config: String,
        rule_config_tuple: String,
        has_hash_map: bool,
        has_hash_set: bool,
    ) -> Self {
        self.rule_config = Some(rule_config);
        self.rule_config_tuple = Some(rule_config_tuple);
        self.has_hash_map = has_hash_map;
        self.has_hash_set = has_hash_set;
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
                    case.with_group_comment(group_comment.clone())
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
                if let Some(Expression::ObjectExpression(obj_expr)) = &export_decl
                    .declaration
                    .as_expression()
                    .map(oxc_ast::ast::Expression::get_inner_expression)
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
            if ident.name == "describe"
                && let Some(Argument::StringLiteral(lit)) = expr.arguments.first()
            {
                pushed = true;
                self.group_comment_stack.push(lit.value.to_string());
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

                if let Expression::CallExpression(call_expr) = &prop.value
                    && call_expr.callee.is_member_expression()
                {
                    // for eslint-plugin-react
                    if let Some(Argument::ArrayExpression(array_expr)) = call_expr.arguments.first()
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
                if let Expression::CallExpression(call_expr) = &prop.value
                    && call_expr.callee.is_member_expression()
                    && let Some(Argument::ArrayExpression(array_expr)) = call_expr.arguments.first()
                {
                    let array_expr = self.alloc(array_expr);
                    for arg in &array_expr.elements {
                        if let Some(expr) = arg.as_expression() {
                            self.add_invalid_test(expr);
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
        if let Expression::Identifier(iden) = object
            && iden.name == "parsers"
            && property.name == "all"
            && let Some(arg) = call_expr.arguments.first()
        {
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
        expr = object;
    }
}

#[derive(Debug, Serialize, PartialEq)]
enum RuleConfigElement {
    Enum(Vec<RuleConfigElement>),
    /// Object maps property name -> (element, optional default literal string)
    Object(FxHashMap<String, (RuleConfigElement, Option<String>)>),
    Map(Box<RuleConfigElement>),
    Array(Box<RuleConfigElement>),
    Set(Box<RuleConfigElement>),
    Boolean,
    StringLiteral(String),
    String,
    Number,
    Integer,
    Nullable(Box<RuleConfigElement>),
    True,
    False,
    Null,
}

struct RuleConfigOutput {
    seen_names: FxHashSet<String>,
    output: String,
    has_errors: bool,
    log_errors: bool,
    has_hash_map: bool,
    has_hash_set: bool,
}

impl RuleConfigOutput {
    fn new(log_errors: bool) -> Self {
        Self {
            seen_names: FxHashSet::default(),
            output: String::new(),
            has_errors: false,
            has_hash_map: false,
            has_hash_set: false,
            log_errors,
        }
    }

    fn log_error(&mut self, message: &str) {
        if self.log_errors {
            println!("\x1b[31m[ERROR]\x1b[0m: {message}");
        }
        self.has_errors = true;
    }

    fn escape_rust_identifier(ident: &str) -> String {
        // List of Rust reserved keywords that cannot be used as identifiers directly.
        // We use raw identifiers `r#foo` for those.
        const RUST_KEYWORDS: [&str; 41] = [
            "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
            "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
            "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
            "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become",
            "box",
        ];
        if RUST_KEYWORDS.contains(&ident) { format!("r#{ident}") } else { ident.to_string() }
    }

    fn extract_output(&mut self, element: &RuleConfigElement, field_name: &str) -> Option<String> {
        let (element_label, element_output) = self.extract_output_inner(element, field_name)?;
        if let Some(element_output) = element_output {
            let _ = writeln!(self.output, "\n{element_output}");
        }
        Some(element_label)
    }

    fn extract_output_inner(
        &mut self,
        element: &RuleConfigElement,
        field_name: &str,
    ) -> Option<(String, Option<String>)> {
        match element {
            RuleConfigElement::Enum(elements) => {
                let enum_name = field_name.to_case(Case::Pascal);
                let enum_name = if self.seen_names.contains(&enum_name) {
                    let mut iteration = 0;
                    loop {
                        let enum_name = format!("{enum_name}{iteration}");
                        if !self.seen_names.contains(&enum_name) {
                            break enum_name;
                        }
                        iteration += 1;
                    }
                } else {
                    enum_name
                };
                self.seen_names.insert(enum_name.clone());
                let mut output = String::new();
                output.push_str(
                    "#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]\n",
                );
                // Determine if all enum string values are kebab-case. If so, prefer
                // using `rename_all = "kebab-case"` instead of per-variant renames.
                let string_values: Vec<&String> = elements
                    .iter()
                    .filter_map(|e| {
                        if let RuleConfigElement::StringLiteral(s) = e { Some(s) } else { None }
                    })
                    .collect();
                let all_strings =
                    !string_values.is_empty() && string_values.len() == elements.len();
                let use_kebab =
                    all_strings && string_values.iter().all(|s| *s == &s.to_case(Case::Kebab));
                let rename_style = if use_kebab { "kebab-case" } else { "camelCase" };
                if all_strings {
                    // When all variants are string literals, `untagged` is unnecessary
                    let _ = writeln!(output, "#[serde(rename_all = \"{rename_style}\")]");
                } else {
                    // Non-string variants require `untagged` to allow multiple shapes
                    let _ = writeln!(output, "#[serde(untagged, rename_all = \"{rename_style}\")]");
                }
                let _ = writeln!(output, "enum {enum_name} {{");
                let mut unlabeled_enum_value_count = 0;
                let mut added_default = false;
                let (enum_fields, fields_output) = elements
                    .iter()
                    .filter_map(|element| match element {
                        RuleConfigElement::StringLiteral(string_literal) => {
                            let is_valid_identifier_regex = regex!(r"^[a-zA-Z_]\w*$");
                            let formatted_string_literal = string_literal.to_case(Case::Pascal);
                            if is_valid_identifier_regex.is_match(&formatted_string_literal) {
                                // If using kebab-case for all variants, we can omit per-variant
                                // rename attributes (they're covered by rename_all). Otherwise
                                // fall back to the existing per-variant rename behavior.
                                let rename = if use_kebab || formatted_string_literal.to_case(Case::Camel)
                                    == *string_literal
                                {
                                    None
                                } else {
                                    Some(format!("rename = \"{string_literal}\""))
                                };
                                Some((rename, Some(formatted_string_literal), None, None))
                            } else {
                                // Non-identifier variant names always need an explicit rename
                                Some((
                                    Some(format!("rename = \"{string_literal}\"")),
                                    None,
                                    None,
                                    None,
                                ))
                            }
                        }
                        RuleConfigElement::Object(_)
                        | RuleConfigElement::Array(_)
                        | RuleConfigElement::Set(_)
                        | RuleConfigElement::Nullable(_)
                        | RuleConfigElement::Boolean
                        | RuleConfigElement::String
                        | RuleConfigElement::Number
                        | RuleConfigElement::Integer
                        | RuleConfigElement::Enum(_)
                        | RuleConfigElement::Map(_) => {
                            let (element_label, element_output) =
                                self.extract_output_inner(element, field_name)?;
                            Some((
                                None,
                                None,
                                Some(element_label),
                                element_output,
                            ))
                        }
                        RuleConfigElement::True
                        | RuleConfigElement::False
                        | RuleConfigElement::Null => {
                            self.log_error(&format!("Unhandled enum element: {element:?}"));
                            None
                        }
                    })
                    .fold(
                        (String::new(), String::new()),
                        |(mut enum_fields, mut enum_value_output),
                         (schemars_tag, enum_label, enum_value, element_output)| {
                            let mut schemars_tags = vec![];
                            if let Some(serde_tag) = schemars_tag {
                                schemars_tags.push(serde_tag);
                            }
                            if !schemars_tags.is_empty() {
                                let _ = writeln!(
                                    enum_fields,
                                    "    #[serde({})]",
                                    schemars_tags.join(", ")
                                );
                            }
                            // Bogus default tag, but allows generated code to compile.
                            // Complicated enum values need a default value, so only add the bogus tag
                            // if it's easy.
                            if enum_value.is_none() && !added_default {
                                let _ = writeln!(enum_fields, "    #[default]");
                                added_default = true;
                            }
                            let enum_label = enum_label.unwrap_or_else(|| {
                                unlabeled_enum_value_count += 1;
                                format!("Unlabeled{unlabeled_enum_value_count}")
                            });
                            let _ = writeln!(enum_fields, "    {enum_label}{},", if let Some(enum_value) = enum_value { format!("({enum_value})") } else { String::new() });
                            if let Some(element_output) = element_output {
                                let _ = writeln!(enum_value_output, "\n{element_output}");
                            }
                            (enum_fields, enum_value_output)
                        },
                    );

                let _ = writeln!(output, "{enum_fields}\n}}\n{fields_output}\n");
                Some((enum_name, Some(output)))
            }
            RuleConfigElement::Object(hash_map) => {
                let struct_name = if self.seen_names.contains(&field_name.to_case(Case::Pascal)) {
                    let mut iteration = 0;
                    loop {
                        let struct_name =
                            format!("{}{iteration}", field_name.to_case(Case::Pascal));
                        if !self.seen_names.contains(&struct_name) {
                            break struct_name;
                        }
                        iteration += 1;
                    }
                } else {
                    field_name.to_case(Case::Pascal)
                };
                self.seen_names.insert(struct_name.clone());
                let mut fields_output = String::new();
                let mut field_entries: Vec<(String, String, String, Option<String>)> = Vec::new();
                for (raw_key, (value, default)) in hash_map {
                    let key_pascal = raw_key.to_case(Case::Pascal);
                    let key_snake = key_pascal.to_case(Case::Snake);
                    let Some((value_label, value_output)) =
                        self.extract_output_inner(value, &key_pascal)
                    else {
                        continue;
                    };
                    field_entries.push((
                        raw_key.clone(),
                        key_snake,
                        value_label.clone(),
                        default.clone(),
                    ));
                    if let Some(value_output) = value_output {
                        let _ = writeln!(fields_output, "{value_output}\n");
                    }
                }
                let has_defaults = field_entries.iter().any(|(_, _, _, default)| default.is_some());
                let mut output = String::new();
                if has_defaults {
                    output
                        .push_str("#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]\n");
                } else {
                    output.push_str(
                        "#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]\n",
                    );
                }
                // Add a default in serde.
                output.push_str("#[serde(rename_all = \"camelCase\", default)]\n");
                let _ = writeln!(output, "struct {struct_name} {{");
                for (raw_key, key_snake, value_label, _) in &field_entries {
                    if key_snake.to_case(Case::Camel) != *raw_key {
                        let _ = writeln!(output, "    #[serde(rename = \"{raw_key}\")]");
                    }
                    // Add a placeholder documentation comment for the field so maintainers remember to
                    // describe the purpose of each generated config field.
                    let _ = writeln!(output, "    /// FIXME: Describe the purpose of this field.");
                    let escaped_key_snake = Self::escape_rust_identifier(key_snake);
                    let _ = writeln!(output, "    {escaped_key_snake}: {value_label},");
                }
                let _ = writeln!(output, "}}\n{fields_output}");

                if has_defaults {
                    let mut impl_output = String::new();
                    let _ = writeln!(impl_output, "impl Default for {struct_name} {{");
                    let _ = writeln!(impl_output, "    fn default() -> Self {{");
                    let _ = writeln!(impl_output, "        Self {{");
                    for (raw_key, key_snake, value_label, default) in &field_entries {
                        let escaped_key_snake = Self::escape_rust_identifier(key_snake);
                        let field_value = if let Some(default_json) = default {
                            if value_label.starts_with("Option<") {
                                let inner = &value_label[7..value_label.len() - 1];
                                let s = default_json.trim();
                                if s == "null" {
                                    "None".to_string()
                                } else if let Some(lit) =
                                    Self::render_default_literal(default_json, inner)
                                {
                                    format!("Some({lit})")
                                } else {
                                    self.log_error(&format!("Failed to render default for field {raw_key} - using Default::default()"));
                                    "Default::default()".to_string()
                                }
                            } else if let Some(lit) =
                                Self::render_default_literal(default_json, value_label)
                            {
                                lit
                            } else {
                                self.log_error(&format!("Failed to render default for field {raw_key} - using Default::default()"));
                                "Default::default()".to_string()
                            }
                        } else {
                            "Default::default()".to_string()
                        };
                        let _ = writeln!(
                            impl_output,
                            "            {escaped_key_snake}: {field_value},"
                        );
                    }
                    let _ = writeln!(impl_output, "        }}");
                    let _ = writeln!(impl_output, "    }}");
                    let _ = writeln!(impl_output, "}}\n");

                    output.push_str(&impl_output);
                }

                Some((struct_name, Some(output)))
            }
            RuleConfigElement::Array(element) => {
                let (element_label, element_output) =
                    self.extract_output_inner(element, field_name)?;
                Some((format!("Vec<{element_label}>"), element_output))
            }
            RuleConfigElement::Set(element) => {
                self.has_hash_set = true;
                let (element_label, element_output) =
                    self.extract_output_inner(element, field_name)?;
                Some((format!("FxHashSet<{element_label}>"), element_output))
            }
            RuleConfigElement::Integer => Some((String::from("i32"), None)),
            RuleConfigElement::String => Some((String::from("String"), None)),
            RuleConfigElement::Number => Some((String::from("f32"), None)),
            RuleConfigElement::Boolean => Some((String::from("bool"), None)),
            RuleConfigElement::Nullable(element) => {
                let (element_label, element_output) =
                    self.extract_output_inner(element, field_name)?;
                Some((format!("Option<{element_label}>"), element_output))
            }
            RuleConfigElement::Map(element) => {
                let (element_label, element_output) =
                    self.extract_output_inner(element, field_name)?;
                self.has_hash_map = true;
                Some((format!("FxHashMap<String, {element_label}>"), element_output))
            }
            RuleConfigElement::StringLiteral(_)
            | RuleConfigElement::True
            | RuleConfigElement::False
            | RuleConfigElement::Null => {
                self.log_error(&format!("Unhandled element for output: {element:?}"));
                None
            }
        }
    }

    fn render_default_literal(default_json: &str, typ: &str) -> Option<String> {
        let s = default_json.trim();
        if s.starts_with('"') && s.ends_with('"') {
            // JSON string literal - use as Rust String literal
            return Some(format!("String::from({s})"));
        }
        if s == "true" || s == "false" {
            return Some(s.to_string());
        }
        if s == "null" {
            if typ.starts_with("Option<") {
                return Some("None".to_string());
            }
            return Some("Default::default()".to_string());
        }
        // Simple number check (integer or float)
        if s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-') {
            if typ == "i32" {
                return Some(s.to_string());
            }
            if typ == "f32" {
                return Some(format!("{s}f32"));
            }
            return Some(s.to_string());
        }
        // Array default handling
        if s.starts_with('[') && s.ends_with(']') {
            // Only support arrays when expected type is Vec<...>
            if let Some(inner) = typ.strip_prefix("Vec<").and_then(|t| t.strip_suffix('>')) {
                // Convert possible JS-like literal to JSON
                let json_text = json::convert_config_to_json_literal(s);
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json_text)
                    && let Some(arr) = value.as_array()
                {
                    let mut elems = Vec::new();
                    for item in arr {
                        if let Ok(item_json) = serde_json::to_string(item)
                            && let Some(lit) = Self::render_default_literal(&item_json, inner)
                        {
                            elems.push(lit);
                            continue;
                        }
                        // failed to render an element -> unsupported
                        return None;
                    }
                    return Some(format!("vec![{}]", elems.join(", ")));
                }
            }
        }

        // Complex structures (objects) are unsupported for now
        None
    }
}

struct RuleConfig<'a> {
    elements: Vec<RuleConfigElement>,
    next_element: Option<RuleConfigElement>,
    source_text: &'a str,
    property_default: Option<String>,
    has_errors: bool,
    log_errors: bool,
}

impl<'a> RuleConfig<'a> {
    fn new(source_text: &'a str, log_errors: bool) -> Self {
        Self {
            elements: vec![],
            next_element: None,
            source_text,
            property_default: None,
            has_errors: false,
            log_errors,
        }
    }

    fn log_error(&mut self, message: &str) {
        if self.log_errors {
            println!("\x1b[31m[ERROR]\x1b[0m: {message}");
        }
        self.has_errors = true;
    }

    // Helper function to handle 'type' property
    fn handle_type_property(&mut self, value: &Expression<'a>) -> Option<RuleConfigElement> {
        match value {
            Expression::StringLiteral(lit) => self.parse_type_string_literal(lit),
            Expression::ArrayExpression(array_expression) => {
                self.parse_type_array_expression(array_expression)
            }
            _ => {
                self.log_error(&format!(
                    "Unhandled `type` expression: {}",
                    value.span().source_text(self.source_text)
                ));
                None
            }
        }
    }

    // Helper function to parse type string literals
    fn parse_type_string_literal(&mut self, lit: &StringLiteral) -> Option<RuleConfigElement> {
        match lit.value.as_str() {
            "string" => Some(RuleConfigElement::String),
            "boolean" => Some(RuleConfigElement::Boolean),
            "number" => Some(RuleConfigElement::Number),
            "integer" => Some(RuleConfigElement::Integer),
            // For loose schemas without `items`/`properties`, treat `array` as array of strings
            // and `object` as a map of string values so we can still generate a usable config.
            "array" => Some(RuleConfigElement::Array(Box::new(RuleConfigElement::String))),
            "object" => Some(RuleConfigElement::Map(Box::new(RuleConfigElement::String))),
            _ => {
                self.log_error(&format!("Unhandled `type` value: {}", lit.value));
                None
            }
        }
    }

    // Helper function to parse type array expressions
    fn parse_type_array_expression(
        &mut self,
        array_expression: &ArrayExpression<'a>,
    ) -> Option<RuleConfigElement> {
        if array_expression.elements.len() != 2 {
            if array_expression.elements.len() != 1 {
                self.log_error(&format!(
                    "Unhandled `type` expression: {}",
                    array_expression.span().source_text(self.source_text)
                ));
                return None;
            }
            let ArrayExpressionElement::StringLiteral(literal) = &array_expression.elements[0]
            else {
                self.log_error(&format!(
                    "Unhandled `type` expression: {}",
                    array_expression.span().source_text(self.source_text)
                ));
                return None;
            };
            let element = self.parse_type_string_literal(literal)?;
            return Some(RuleConfigElement::Nullable(Box::new(element)));
        }
        let first_element = &array_expression.elements[0];
        let second_element = &array_expression.elements[1];
        let ArrayExpressionElement::StringLiteral(first_literal) = first_element else {
            self.log_error(&format!(
                "Unhandled `type` expression: {}",
                array_expression.span().source_text(self.source_text)
            ));
            return None;
        };
        let ArrayExpressionElement::StringLiteral(second_literal) = second_element else {
            self.log_error(&format!(
                "Unhandled `type` expression: {}",
                array_expression.span().source_text(self.source_text)
            ));
            return None;
        };
        if (first_literal.value == "null") == (second_literal.value == "null") {
            self.log_error(&format!(
                "Unhandled `type` expression: {}",
                array_expression.span().source_text(self.source_text)
            ));
            return None;
        }
        let non_null_literal =
            if first_literal.value == "null" { second_literal } else { first_literal };
        let nested_element = self.parse_type_string_literal(non_null_literal)?;
        Some(RuleConfigElement::Nullable(Box::new(nested_element)))
    }

    // Helper function to extract properties
    fn extract_properties(
        &mut self,
        object_expression: &ObjectExpression<'a>,
    ) -> FxHashMap<String, (RuleConfigElement, Option<String>)> {
        let mut properties: FxHashMap<String, (RuleConfigElement, Option<String>)> =
            FxHashMap::default();
        for object_property_kind in &object_expression.properties {
            let ObjectPropertyKind::ObjectProperty(object_property) = &object_property_kind else {
                self.log_error(&format!(
                    "Cannot parse object property kind: {}",
                    object_property_kind.span().source_text(self.source_text)
                ));
                continue;
            };
            let PropertyKey::StaticIdentifier(identifier) = &object_property.key else {
                self.log_error(&format!(
                    "Cannot parse object property key: {}",
                    object_property.key.span().source_text(self.source_text)
                ));
                continue;
            };
            let Expression::ObjectExpression(object_expression) = &object_property.value else {
                self.log_error(&format!(
                    "Cannot parse object property value: {}",
                    object_property.value.span().source_text(self.source_text)
                ));
                continue;
            };
            // reset property_default before parsing
            self.property_default = None;
            self.visit_object_expression(object_expression);
            let Some(element) = self.next_element.take() else {
                self.log_error(&String::from("Cannot find next element"));
                continue;
            };
            let default = self.property_default.take();
            properties.insert(identifier.name.into(), (element, default));
        }
        properties
    }

    // Helper function to extract enum elements
    fn extract_enum_elements(
        &mut self,
        array_expression: &ArrayExpression<'a>,
    ) -> Vec<RuleConfigElement> {
        array_expression
            .elements
            .iter()
            .filter_map(|arg| match arg {
                ArrayExpressionElement::StringLiteral(string_literal) => {
                    Some(RuleConfigElement::StringLiteral(string_literal.value.into()))
                }
                ArrayExpressionElement::BooleanLiteral(boolean_literal) => {
                    if boolean_literal.value {
                        Some(RuleConfigElement::True)
                    } else {
                        Some(RuleConfigElement::False)
                    }
                }
                ArrayExpressionElement::NullLiteral(_) => Some(RuleConfigElement::Null),
                _ => {
                    self.log_error(&format!(
                        "Cannot parse `enum` value: {}",
                        arg.span().source_text(self.source_text)
                    ));
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    // Helper function to extract 'anyOf' or 'oneOf' elements
    fn extract_any_of_elements(
        &mut self,
        array_expression: &ArrayExpression<'a>,
        identifier: &IdentifierName,
    ) -> Vec<RuleConfigElement> {
        let mut elements = Vec::new();
        for arg in &array_expression.elements {
            let ArrayExpressionElement::ObjectExpression(object_expression) = arg else {
                self.log_error(&format!(
                    "Cannot parse `{}` value: {}",
                    identifier.name,
                    arg.span().source_text(self.source_text)
                ));
                continue;
            };
            self.visit_object_expression(object_expression);
            let Some(element) = self.next_element.take() else {
                return elements;
            };
            match element {
                RuleConfigElement::Enum(child_elements) => {
                    elements.extend(child_elements);
                }
                _ => {
                    elements.push(element);
                }
            }
        }
        elements
    }
}

impl<'a> Visit<'a> for RuleConfig<'a> {
    fn visit_program(&mut self, program: &Program<'a>) {
        for stmt in &program.body {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &Statement<'a>) {
        // handle `export default { meta: { schema: ... } }` (ESM/TS)
        if let Statement::ExportDefaultDeclaration(export_decl) = stmt {
            // First, handle the simple case: `export default { meta: { schema: ... } }`
            if let Some(Expression::ObjectExpression(object_expression)) = &export_decl
                .declaration
                .as_expression()
                .map(oxc_ast::ast::Expression::get_inner_expression)
            {
                for object_property_kind in &object_expression.properties {
                    let ObjectPropertyKind::ObjectProperty(object_property) = &object_property_kind
                    else {
                        continue;
                    };
                    let PropertyKey::StaticIdentifier(identifier) = &object_property.key else {
                        continue;
                    };
                    if identifier.name != "meta" {
                        continue;
                    }
                    let Expression::ObjectExpression(meta_obj) = &object_property.value else {
                        continue;
                    };
                    for object_property_kind in &meta_obj.properties {
                        let ObjectPropertyKind::ObjectProperty(object_property) =
                            &object_property_kind
                        else {
                            continue;
                        };
                        let PropertyKey::StaticIdentifier(identifier) = &object_property.key else {
                            continue;
                        };
                        if identifier.name != "schema" {
                            continue;
                        }
                        match &object_property.value {
                            Expression::ArrayExpression(array_expression) => {
                                self.elements = array_expression
                                    .elements
                                    .iter()
                                    .filter_map(|element| {
                                        let ArrayExpressionElement::ObjectExpression(
                                            object_expression,
                                        ) = element
                                        else {
                                            return None;
                                        };
                                        self.next_element = None;
                                        self.visit_object_expression(object_expression);
                                        self.next_element.take()
                                    })
                                    .collect::<Vec<_>>();
                            }
                            Expression::ObjectExpression(object_expression) => {
                                self.visit_object_expression(object_expression);
                                let Some(element) = self.next_element.take() else { return };
                                self.elements = vec![element];
                            }
                            _ => {
                                self.log_error(&format!(
                                    "Cannot parse `schema` value: {}",
                                    object_property.value.span().source_text(self.source_text)
                                ));
                            }
                        }
                        return;
                    }
                }
            }

            // Otherwise: handle `export default createRule({ meta: { schema: ... } })` style
            if let Some(Expression::CallExpression(call_expr)) = &export_decl
                .declaration
                .as_expression()
                .map(oxc_ast::ast::Expression::get_inner_expression)
            {
                for arg in &call_expr.arguments {
                    if !arg.is_expression() {
                        continue;
                    }
                    let Some(expr) = arg.as_expression() else { continue };
                    let Expression::ObjectExpression(object_expression) = expr else { continue };
                    for object_property_kind in &object_expression.properties {
                        let ObjectPropertyKind::ObjectProperty(object_property) =
                            &object_property_kind
                        else {
                            continue;
                        };
                        let PropertyKey::StaticIdentifier(identifier) = &object_property.key else {
                            continue;
                        };
                        if identifier.name != "meta" {
                            continue;
                        }
                        let Expression::ObjectExpression(meta_obj) = &object_property.value else {
                            continue;
                        };
                        for object_property_kind in &meta_obj.properties {
                            let ObjectPropertyKind::ObjectProperty(object_property) =
                                &object_property_kind
                            else {
                                continue;
                            };
                            let PropertyKey::StaticIdentifier(identifier) = &object_property.key
                            else {
                                continue;
                            };
                            if identifier.name != "schema" {
                                continue;
                            }
                            match &object_property.value {
                                Expression::ArrayExpression(array_expression) => {
                                    self.elements = array_expression
                                        .elements
                                        .iter()
                                        .filter_map(|element| {
                                            let ArrayExpressionElement::ObjectExpression(
                                                object_expression,
                                            ) = element
                                            else {
                                                return None;
                                            };
                                            self.next_element = None;
                                            self.visit_object_expression(object_expression);
                                            self.next_element.take()
                                        })
                                        .collect::<Vec<_>>();
                                }
                                Expression::ObjectExpression(object_expression) => {
                                    self.visit_object_expression(object_expression);
                                    let Some(element) = self.next_element.take() else { return };
                                    self.elements = vec![element];
                                }
                                _ => {
                                    self.log_error(&format!(
                                        "Cannot parse `schema` value: {}",
                                        object_property.value.span().source_text(self.source_text)
                                    ));
                                }
                            }
                            return;
                        }
                    }
                }
            }
        }

        // handle `export const meta = { schema: ... }` (named export)
        if let Statement::ExportNamedDeclaration(export_decl) = stmt
            && let Some(Declaration::VariableDeclaration(var_decl)) =
                export_decl.declaration.as_ref()
        {
            for declarator in &var_decl.declarations {
                let BindingPattern::BindingIdentifier(binding_ident) = &declarator.id else {
                    continue;
                };
                if binding_ident.name != "meta" {
                    continue;
                }
                let Some(init) = &declarator.init else { continue };
                if let Expression::ObjectExpression(object_expression) = init {
                    for object_property_kind in &object_expression.properties {
                        let ObjectPropertyKind::ObjectProperty(object_property) =
                            &object_property_kind
                        else {
                            continue;
                        };
                        let PropertyKey::StaticIdentifier(identifier) = &object_property.key else {
                            continue;
                        };
                        if identifier.name != "schema" {
                            continue;
                        }
                        match &object_property.value {
                            Expression::ArrayExpression(array_expression) => {
                                self.elements = array_expression
                                    .elements
                                    .iter()
                                    .filter_map(|element| {
                                        let ArrayExpressionElement::ObjectExpression(
                                            object_expression,
                                        ) = element
                                        else {
                                            return None;
                                        };
                                        self.next_element = None;
                                        self.visit_object_expression(object_expression);
                                        self.next_element.take()
                                    })
                                    .collect::<Vec<_>>();
                            }
                            Expression::ObjectExpression(object_expression) => {
                                self.visit_object_expression(object_expression);
                                let Some(element) = self.next_element.take() else { return };
                                self.elements = vec![element];
                            }
                            _ => {
                                self.log_error(&format!(
                                    "Cannot parse `schema` value: {}",
                                    object_property.value.span().source_text(self.source_text)
                                ));
                            }
                        }
                        return;
                    }
                }
            }
        }

        // fall back to CommonJS `module.exports = { meta: { schema: ... } }`
        let Statement::ExpressionStatement(expression_statement) = stmt else {
            return;
        };
        let Expression::AssignmentExpression(assignment_expression) =
            &expression_statement.expression
        else {
            return;
        };
        let AssignmentTarget::StaticMemberExpression(static_member_expression) =
            &assignment_expression.left
        else {
            return;
        };
        let Expression::Identifier(identifier) = &static_member_expression.object else {
            return;
        };
        if identifier.name != "module" {
            return;
        }
        if static_member_expression.property.name != "exports" {
            return;
        }
        let Expression::ObjectExpression(object_expression) = &assignment_expression.right else {
            return;
        };
        for object_property_kind in &object_expression.properties {
            let ObjectPropertyKind::ObjectProperty(object_property) = &object_property_kind else {
                continue;
            };
            let PropertyKey::StaticIdentifier(identifier) = &object_property.key else {
                continue;
            };
            if identifier.name != "meta" {
                continue;
            }
            let Expression::ObjectExpression(object_expression) = &object_property.value else {
                continue;
            };
            for object_property_kind in &object_expression.properties {
                let ObjectPropertyKind::ObjectProperty(object_property) = &object_property_kind
                else {
                    continue;
                };
                let PropertyKey::StaticIdentifier(identifier) = &object_property.key else {
                    continue;
                };
                if identifier.name != "schema" {
                    continue;
                }
                match &object_property.value {
                    Expression::ArrayExpression(array_expression) => {
                        self.elements = array_expression
                            .elements
                            .iter()
                            .filter_map(|element| {
                                let ArrayExpressionElement::ObjectExpression(object_expression) =
                                    element
                                else {
                                    return None;
                                };
                                self.next_element = None;
                                self.visit_object_expression(object_expression);
                                self.next_element.take()
                            })
                            .collect::<Vec<_>>();
                    }
                    Expression::ObjectExpression(object_expression) => {
                        self.visit_object_expression(object_expression);
                        let Some(element) = self.next_element.take() else {
                            return;
                        };
                        self.elements = vec![element];
                    }
                    _ => {
                        self.log_error(&format!(
                            "Cannot parse `schema` value: {}",
                            object_property.value.span().source_text(self.source_text)
                        ));
                    }
                }
                return;
            }
        }
    }

    fn visit_object_expression(&mut self, object_expression: &ObjectExpression<'a>) {
        let mut rule_config_element = None;
        let mut is_unique = false;
        for object_property_kind in &object_expression.properties {
            let ObjectPropertyKind::ObjectProperty(object_property) = &object_property_kind else {
                self.log_error(&format!(
                    "Cannot parse object property kind: {}",
                    object_property_kind.span().source_text(self.source_text)
                ));
                continue;
            };
            let PropertyKey::StaticIdentifier(identifier) = &object_property.key else {
                self.log_error(&format!(
                    "Cannot parse object property key: {}",
                    object_property.key.span().source_text(self.source_text)
                ));
                continue;
            };
            match identifier.name.as_str() {
                "type" => {
                    rule_config_element = self.handle_type_property(&object_property.value);
                }
                "properties" => {
                    let Expression::ObjectExpression(object_expression) = &object_property.value
                    else {
                        self.log_error(&format!(
                            "Cannot parse `properties` value: {}",
                            object_property.value.span().source_text(self.source_text)
                        ));
                        continue;
                    };
                    let properties = self.extract_properties(object_expression);
                    rule_config_element = Some(RuleConfigElement::Object(properties));
                }
                "items" => {
                    let Expression::ObjectExpression(object_expression) = &object_property.value
                    else {
                        self.log_error(&format!(
                            "Cannot parse `items` value: {}",
                            object_property.value.span().source_text(self.source_text)
                        ));
                        continue;
                    };
                    self.visit_object_expression(object_expression);
                    let Some(element) = self.next_element.take() else {
                        self.log_error(&String::from("Cannot find next element"));
                        continue;
                    };
                    if is_unique {
                        rule_config_element = Some(RuleConfigElement::Set(Box::new(element)));
                    } else {
                        rule_config_element = Some(RuleConfigElement::Array(Box::new(element)));
                    }
                }
                "uniqueItems" => {
                    let Expression::BooleanLiteral(boolean_literal) = &object_property.value else {
                        self.log_error(&format!(
                            "Cannot parse `uniqueItems` value: {}",
                            object_property.value.span().source_text(self.source_text)
                        ));
                        continue;
                    };
                    if !boolean_literal.value {
                        continue;
                    }
                    is_unique = true;
                    let Some(RuleConfigElement::Array(element)) = rule_config_element else {
                        continue;
                    };
                    rule_config_element = Some(RuleConfigElement::Set(element));
                }
                "enum" => {
                    let Expression::ArrayExpression(array_expression) = &object_property.value
                    else {
                        self.log_error(&format!(
                            "Cannot parse `enum` values: {}",
                            object_property.value.span().source_text(self.source_text)
                        ));
                        continue;
                    };
                    let elements = self.extract_enum_elements(array_expression);
                    rule_config_element = Some(RuleConfigElement::Enum(elements));
                }
                "anyOf" | "oneOf" => {
                    let Expression::ArrayExpression(array_expression) = &object_property.value
                    else {
                        self.log_error(&format!(
                            "Cannot parse `{}` value: {}",
                            identifier.name,
                            object_property.value.span().source_text(self.source_text)
                        ));
                        continue;
                    };
                    let elements = self.extract_any_of_elements(array_expression, identifier);
                    rule_config_element = Some(RuleConfigElement::Enum(elements));
                }
                "additionalProperties" => match &object_property.value {
                    Expression::ObjectExpression(object_expression) => {
                        self.visit_object_expression(object_expression);
                        let Some(element) = self.next_element.take() else {
                            self.log_error(&String::from("Cannot find next element"));
                            continue;
                        };
                        rule_config_element = Some(RuleConfigElement::Map(Box::new(element)));
                    }
                    Expression::BooleanLiteral(boolean_literal) => {
                        if boolean_literal.value {
                            self.log_error(&format!(
                                "Unhandled `additionalProperties` value: {}",
                                object_property.value.span().source_text(self.source_text)
                            ));
                        }
                    }
                    _ => {
                        self.log_error(&format!(
                            "Unhandled `additionalProperties` value: {}",
                            object_property.value.span().source_text(self.source_text)
                        ));
                    }
                },
                "default" => {
                    // Capture default literal (JSON text) for later use when generating Default impl
                    self.property_default = Some(
                        object_property.value.span().source_text(self.source_text).to_string(),
                    );
                }
                "required" | "minItems" | "minimum" | "minLength" | "maxItems"
                | "minProperties" | "maximum" | "pattern" => {}
                _ => {
                    self.log_error(&format!("Unhandled key `{}`", identifier.name));
                }
            }
        }
        self.next_element = rule_config_element;
    }
}

/// Result of parsing a test file containing pass/fail test cases.
struct ParsedTestFile {
    pass_cases: Vec<TestCase>,
    fail_cases: Vec<TestCase>,
}

impl ParsedTestFile {
    /// Returns whether any test cases have a config.
    fn has_config(&self) -> bool {
        self.pass_cases.iter().any(|case| case.config.is_some())
            || self.fail_cases.iter().any(|case| case.config.is_some())
    }

    /// Returns whether any test cases have settings.
    fn has_settings(&self) -> bool {
        self.pass_cases.iter().any(|case| case.settings.is_some())
            || self.fail_cases.iter().any(|case| case.settings.is_some())
    }

    /// Returns whether any test cases have a filename.
    fn has_filename(&self) -> bool {
        self.pass_cases.iter().any(|case| case.filename.is_some())
            || self.fail_cases.iter().any(|case| case.filename.is_some())
    }
}

/// Parses a test file and extracts pass/fail test cases.
///
/// # Arguments
/// * `source_text` - The source code of the test file
/// * `file_path` - The path to the test file (used to determine source type)
///
/// # Returns
/// A `ParsedTestFile` containing the extracted test cases, or an error message.
fn parse_test_file(source_text: &str, file_path: &str) -> Result<ParsedTestFile, String> {
    let allocator = Allocator::default();
    let source_type =
        SourceType::from_path(file_path).map_err(|e| format!("Invalid file path: {e:?}"))?;
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    if !ret.errors.is_empty() {
        return Err(format!("Parse errors: {:?}", ret.errors));
    }

    let mut state = State::new(source_text);
    state.visit_program(&ret.program);

    Ok(ParsedTestFile { pass_cases: state.pass_cases(), fail_cases: state.fail_cases() })
}

/// Generates case strings from test cases for use in the template.
fn gen_cases_string(
    cases: Vec<TestCase>,
    has_config: bool,
    has_settings: bool,
    has_filename: bool,
) -> (String, String) {
    let mut codes = vec![];
    let mut fix_codes = vec![];
    let mut last_comment = String::new();
    for case in cases {
        let current_comment = case.group_comment();
        let mut code = case.code(has_config, has_settings, has_filename);
        if code.is_empty() {
            continue;
        }
        if let Some(current_comment) = current_comment
            && current_comment != last_comment
        {
            last_comment = current_comment.to_string();
            code = format!(
                "// {}\n{}",
                &last_comment,
                case.code(has_config, has_settings, has_filename)
            );
        }

        if let Some(output) = case.output() {
            fix_codes.push(output);
        }

        codes.push(code);
    }

    (codes.join(",\n"), fix_codes.join(",\n"))
}

/// Builds a `Context` from a parsed test file.
///
/// # Arguments
/// * `parsed` - The parsed test file
/// * `rule_kind` - The kind of rule (e.g., ESLint, Jest, etc.)
/// * `rule_name` - The name of the rule in snake_case
/// * `language` - The language of the test cases (e.g., "js", "ts", "jsx", "tsx")
fn build_context_from_parsed_test(
    parsed: ParsedTestFile,
    rule_kind: RuleKind,
    rule_name: &str,
    language: &str,
) -> Context {
    let has_config = parsed.has_config();
    let has_settings = parsed.has_settings();
    let has_filename = parsed.has_filename();

    let (pass_cases, _) =
        gen_cases_string(parsed.pass_cases, has_config, has_settings, has_filename);
    let (fail_cases, fix_cases) =
        gen_cases_string(parsed.fail_cases, has_config, has_settings, has_filename);

    Context::new(rule_kind, rule_name, pass_cases, fail_cases)
        .with_language(language.to_string())
        .with_filename(has_filename)
        .with_fix_cases(fix_cases)
}

/// Parses a rule source file and extracts the configuration schema.
///
/// # Arguments
/// * `source_text` - The source code of the rule file
/// * `file_path` - The path to the rule file (used to determine source type)
/// * `debug_mode` - Whether to enable debug logging
///
/// # Returns
/// The parsed rule configuration elements, or an error message.
fn parse_rule_source(
    source_text: &str,
    file_path: &str,
    debug_mode: bool,
) -> Result<Vec<RuleConfigElement>, String> {
    let allocator = Allocator::default();
    let source_type =
        SourceType::from_path(file_path).map_err(|e| format!("Invalid file path: {e:?}"))?;
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    if !ret.errors.is_empty() {
        return Err(format!("Parse errors: {:?}", ret.errors));
    }

    let mut config = RuleConfig::new(source_text, debug_mode);

    // TODO: Use the tasks/lint_rules package to get the runtime config object from javascript
    // and parse it here to resolve values of expressions.
    config.visit_program(&ret.program);

    if debug_mode {
        println!("Rule config: {:?}", config.elements);
    }

    Ok(config.elements)
}

/// Applies parsed rule configuration to a context.
///
/// # Arguments
/// * `context` - The context to update
/// * `elements` - The parsed rule configuration elements
/// * `pascal_rule_name` - The Pascal case name of the rule (e.g., "NoLargeSnapshots")
/// * `debug_mode` - Whether to enable debug logging
///
/// # Returns
/// The updated context with rule configuration applied.
fn apply_rule_config_to_context(
    context: Context,
    elements: &[RuleConfigElement],
    pascal_rule_name: &str,
    debug_mode: bool,
) -> Context {
    let mut rule_config_output = RuleConfigOutput::new(debug_mode);
    let config_names = elements
        .iter()
        .enumerate()
        .filter_map(|(index, element)| {
            let element_name = format!(
                "{pascal_rule_name}Config{}",
                if index == 0 { String::new() } else { index.to_string() }
            );
            rule_config_output.extract_output(element, element_name.as_str())
        })
        .collect::<Vec<_>>();

    if debug_mode {
        println!("Rule config names: {config_names:?}");
        println!("Rule Output:\n{}", rule_config_output.output);
    }

    if rule_config_output.has_errors {
        if debug_mode {
            println!("Rule config parsed, but with fatal errors. Not writing config.");
        }
        return context;
    }

    if config_names.is_empty() {
        context
    } else {
        let rule_config_tuple = format!("({})", config_names.join(", "));
        context.with_rule_config(
            rule_config_output.output,
            rule_config_tuple,
            rule_config_output.has_hash_map,
            rule_config_output.has_hash_set,
        )
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
    Promise,
    Vitest,
    Vue,
}

impl TryFrom<&str> for RuleKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "eslint" => Ok(Self::ESLint),
            "jest" => Ok(Self::Jest),
            "typescript" => Ok(Self::Typescript),
            "unicorn" => Ok(Self::Unicorn),
            "import" => Ok(Self::Import),
            "react" => Ok(Self::React),
            "react-perf" => Ok(Self::ReactPerf),
            "jsx-a11y" => Ok(Self::JSXA11y),
            "oxc" => Ok(Self::Oxc),
            "nextjs" => Ok(Self::NextJS),
            "jsdoc" => Ok(Self::JSDoc),
            "n" => Ok(Self::Node),
            "promise" => Ok(Self::Promise),
            "vitest" => Ok(Self::Vitest),
            "vue" => Ok(Self::Vue),
            _ => Err(format!("Invalid `RuleKind`, got `{value}`")),
        }
    }
}

impl Display for RuleKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let kind_name = match self {
            Self::ESLint => "eslint",
            Self::Typescript => "typescript-eslint",
            Self::Jest => "eslint-plugin-jest",
            Self::Unicorn => "eslint-plugin-unicorn",
            Self::Import => "eslint-plugin-import",
            Self::React => "eslint-plugin-react",
            Self::ReactPerf => "eslint-plugin-react-perf",
            Self::JSXA11y => "eslint-plugin-jsx-a11y",
            Self::Oxc => "oxc",
            Self::NextJS => "eslint-plugin-next",
            Self::JSDoc => "eslint-plugin-jsdoc",
            Self::Node => "eslint-plugin-n",
            Self::Promise => "eslint-plugin-promise",
            Self::Vitest => "eslint-plugin-vitest",
            Self::Vue => "eslint-plugin-vue",
        };
        f.write_str(kind_name)
    }
}

fn main() {
    let mut args = std::env::args();
    args.next();

    let rule_name = args.next().expect("expected rule name").to_case(Case::Snake);
    let rule_kind = args.next().map_or(RuleKind::ESLint, |kind| {
        RuleKind::try_from(kind.as_str()).expect("Invalid `RuleKind`")
    });
    let kebab_rule_name = rule_name.to_case(Case::Kebab);
    let camel_rule_name = rule_name.to_case(Case::Camel);
    let pascal_rule_name = rule_name.to_case(Case::Pascal);

    let rule_test_path = match rule_kind {
        RuleKind::ESLint => format!("{ESLINT_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::Jest => format!("{JEST_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::Typescript => format!("{TYPESCRIPT_ESLINT_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::Unicorn => format!("{UNICORN_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::Import => format!("{IMPORT_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::React => format!("{REACT_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::ReactPerf => format!("{REACT_PERF_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::JSXA11y => format!("{JSX_A11Y_TEST_PATH}/{kebab_rule_name}-test.js"),
        RuleKind::NextJS => format!("{NEXT_JS_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::JSDoc => format!("{JSDOC_TEST_PATH}/{camel_rule_name}.js"),
        RuleKind::Node => format!("{NODE_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::Promise => format!("{PROMISE_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::Vitest => format!("{VITEST_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::Vue => format!("{VUE_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::Oxc => String::new(),
    };
    let rule_src_path = match rule_kind {
        RuleKind::ESLint => format!("{ESLINT_RULES_PATH}/{kebab_rule_name}.js"),
        RuleKind::Jest => format!("{JEST_RULES_PATH}/{kebab_rule_name}.ts"),
        RuleKind::Typescript => format!("{TYPESCRIPT_ESLINT_RULES_PATH}/{kebab_rule_name}.ts"),
        RuleKind::Unicorn => format!("{UNICORN_RULES_PATH}/{kebab_rule_name}.js"),
        RuleKind::Import => format!("{IMPORT_RULES_PATH}/{kebab_rule_name}.js"),
        RuleKind::React => format!("{REACT_RULES_PATH}/{kebab_rule_name}.js"),
        RuleKind::ReactPerf => format!("{REACT_PERF_RULES_PATH}/{kebab_rule_name}.js"),
        RuleKind::JSXA11y => format!("{JSX_A11Y_RULES_PATH}/{kebab_rule_name}.js"),
        RuleKind::NextJS => format!("{NEXT_JS_RULES_PATH}/{kebab_rule_name}.js"),
        RuleKind::JSDoc => format!("{JSDOC_RULES_PATH}/{camel_rule_name}.js"),
        RuleKind::Node => format!("{NODE_RULES_PATH}/{kebab_rule_name}.js"),
        RuleKind::Promise => format!("{PROMISE_RULES_PATH}/{kebab_rule_name}.js"),
        RuleKind::Vitest => format!("{VITEST_RULES_PATH}/{kebab_rule_name}.ts"),
        RuleKind::Vue => format!("{VUE_RULES_PATH}/{kebab_rule_name}.js"),
        RuleKind::Oxc => String::new(),
    };
    let language = match rule_kind {
        RuleKind::Typescript | RuleKind::Oxc => "ts",
        RuleKind::NextJS => "tsx",
        RuleKind::React | RuleKind::ReactPerf | RuleKind::JSXA11y => "jsx",
        _ => "js",
    };

    println!("Reading test file from {rule_test_path}");

    let test_body = oxc_tasks_common::agent()
        .get(&rule_test_path)
        .call()
        .map(|mut res| res.body_mut().read_to_string());
    let mut context = match test_body {
        Ok(Ok(body)) => match parse_test_file(&body, &rule_test_path) {
            Ok(parsed) => {
                println!(
                    "File parsed and {} pass cases, {} fail cases are found",
                    parsed.pass_cases.len(),
                    parsed.fail_cases.len()
                );
                build_context_from_parsed_test(parsed, rule_kind, &rule_name, language)
            }
            Err(err) => {
                eprintln!("Failed to parse test file: {err}");
                Context::new(rule_kind, &rule_name, String::new(), String::new())
            }
        },
        Err(err) => {
            println!("Rule tests {rule_name} cannot be found in {rule_kind}, use empty template.");
            println!("Error: {err}");
            Context::new(rule_kind, &rule_name, String::new(), String::new())
        }
        Ok(Err(err)) => {
            println!("Failed to convert rule test code to string: {err}, use empty template");
            Context::new(rule_kind, &rule_name, String::new(), String::new())
        }
    };

    println!("Reading rule source file from {rule_src_path}");

    let rule_src_body = oxc_tasks_common::agent()
        .get(&rule_src_path)
        .call()
        .map(|mut res| res.body_mut().read_to_string());
    match rule_src_body {
        Ok(Ok(body)) => {
            let debug_mode = false;
            match parse_rule_source(&body, &rule_src_path, debug_mode) {
                Ok(elements) => {
                    println!("Rule config parsed.");
                    context = apply_rule_config_to_context(
                        context,
                        &elements,
                        &pascal_rule_name,
                        debug_mode,
                    );
                }
                Err(err) => {
                    eprintln!("Failed to parse rule source: {err}");
                }
            }
        }
        Ok(Err(err)) => {
            println!("Failed to convert rule source code to string: {err}, use empty template");
        }
        Err(err) => {
            println!("Rule source {rule_name} cannot be found in {rule_kind}, use empty template.");
            println!("Error: {err}");
        }
    }

    let rule_name = &context.kebab_rule_name;
    let template = template::Template::with_context(&context);
    if let Err(err) = template.render(rule_kind) {
        eprintln!("failed to render {rule_name} rule template: {err}");
    }

    if let Err(err) = add_rules_entry(&context, rule_kind) {
        eprintln!("failed to add {rule_name} to rules file: {err}");
    }

    if let Err(err) = generate_rule_runner_impl() {
        eprintln!("failed to generate RuleRunner impl for {rule_name}: {err}");
    }
}

fn generate_rule_runner_impl() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::{Command, Stdio};

    println!("Generating RuleRunner impl...");
    let output = Command::new("cargo")
        .args(["run", "-p", "oxc_linter_codegen"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        return Err("Failed to run oxc_linter_codegen".into());
    }

    println!("Formatting generated RuleRunner impl...");
    // format the generated code
    Command::new("cargo").args(["fmt", "--package", "oxc_linter"]).status()?;

    Ok(())
}

fn get_mod_name(rule_kind: RuleKind) -> String {
    match rule_kind {
        RuleKind::ESLint => "eslint".into(),
        RuleKind::Import => "import".into(),
        RuleKind::Typescript => "typescript".into(),
        RuleKind::Jest => "jest".into(),
        RuleKind::React => "react".into(),
        RuleKind::ReactPerf => "react_perf".into(),
        RuleKind::Unicorn => "unicorn".into(),
        RuleKind::JSDoc => "jsdoc".into(),
        RuleKind::JSXA11y => "jsx_a11y".into(),
        RuleKind::Oxc => "oxc".into(),
        RuleKind::NextJS => "nextjs".into(),
        RuleKind::Promise => "promise".into(),
        RuleKind::Vitest => "vitest".into(),
        RuleKind::Node => "node".into(),
        RuleKind::Vue => "vue".into(),
    }
}

/// Adds a module definition for the given rule to the `rules.rs` file, and adds the rule to the
/// `declare_all_lint_rules!` macro block.
fn add_rules_entry(ctx: &Context, rule_kind: RuleKind) -> Result<(), Box<dyn std::error::Error>> {
    let rules_path = "crates/oxc_linter/src/rules.rs";
    let mut rules = std::fs::read_to_string(rules_path)?;

    let mod_name = get_mod_name(rule_kind);
    let mod_def = format!("mod {mod_name}");
    let Some(mod_start) = rules.find(&mod_def) else {
        return Err(format!("failed to find '{mod_def}' in {rules_path}").into());
    };
    let mod_end = &rules[mod_start..]
        .find('}')
        .ok_or(format!("failed to find end of '{mod_def}' module in {rules_path}"))?;
    let mod_rules = &rules[mod_start..(*mod_end + mod_start)];

    // Check if the rule mod def already exists
    let rule_mod_def = format!("pub mod {};", ctx.snake_rule_name);
    let mut needs_mod_insertion = true;

    if mod_rules.contains(&rule_mod_def) {
        needs_mod_insertion = false;
        println!("Rule module '{}' already exists, skipping mod insertion", ctx.snake_rule_name);
    }

    // Insert the rule mod def if it doesn't exist
    if needs_mod_insertion {
        // Find the rule name (`pub mod xyz;`) that comes alphabetically before the new rule mod def,
        // otherwise just append it to the mod.
        let rule_mod_def_start = mod_rules
        .lines()
        .filter_map(|line| line.split_once("pub mod ").map(|(_, rest)| rest))
        .position(|rule_mod| rule_mod < rule_mod_def.as_str())
        .map(|i| i + 1)
        .and_then(|i| rules[mod_start + i..].find("pub mod ").map(|j| i + j))
        .ok_or(format!(
            "failed to find where to insert the new rule mod def ({rule_mod_def}) in {rules_path}"
        ))?;

        rules.insert_str(
            mod_start + rule_mod_def_start,
            &format!("    pub mod {};\n", ctx.snake_rule_name),
        );
    }
    // then, insert `{mod_name}::{rule_name};` in the `declare_all_lint_rules!` macro block
    // in the correct position, alphabetically.
    let declare_all_lint_rules_start = rules
        .find("declare_all_lint_rules!")
        .ok_or(format!("failed to find 'declare_all_lint_rules!' in {rules_path}"))?;
    let rule_def = format!("{mod_name}::{},", ctx.snake_rule_name);
    let mut needs_rule_insertion = true;

    if rules[declare_all_lint_rules_start..].contains(&rule_def) {
        needs_rule_insertion = false;
        println!(
            "Rule '{}::{}' already declared, skipping rule insertion",
            mod_name, ctx.snake_rule_name
        );
    }

    // Insert the rule declaration if it doesn't exist
    if needs_rule_insertion {
        let iter = rules[declare_all_lint_rules_start..]
            .lines()
            .scan(0, |acc, line| {
                let current_offset = *acc;
                *acc += line.len() + 1; // +1 for newline
                Some((current_offset, line))
            })
            .peekable()
            .skip_while(|(_, line)| !line.trim().starts_with(&format!("{mod_name}::")));

        let new_rule = format!("{mod_name}::{}", ctx.snake_rule_name);
        let mut insert_pos = None;

        for (offset, line) in iter {
            let trimmed = line.trim().trim_end_matches(',');
            if !trimmed.starts_with(&format!("{mod_name}::")) {
                // We've reached the next plugin section or the end of the macro
                insert_pos = Some(offset);
                break;
            }

            // Compare alphabetically
            if trimmed > new_rule.as_str() {
                insert_pos = Some(offset);
                break;
            }
        }

        let insert_pos =
            insert_pos.unwrap_or_else(|| rules[declare_all_lint_rules_start..].rfind('}').unwrap());

        let insert_position = declare_all_lint_rules_start + insert_pos - 1;

        rules.insert_str(
            insert_position,
            &format!(
                "\n    {mod_name}::{rule_name},",
                mod_name = mod_name,
                rule_name = ctx.snake_rule_name
            ),
        );
    }

    // Only write if we made changes
    if needs_mod_insertion || needs_rule_insertion {
        std::fs::write(rules_path, rules)?;
        println!("Updated {rules_path}",);
    } else {
        println!("No changes needed - rule already exists in {rules_path}",);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_configs() {
        let mut out = RuleConfigOutput::new(false);
        let mut hm = FxHashMap::default();
        hm.insert("maxItems".to_string(), (RuleConfigElement::Integer, None));
        let element = RuleConfigElement::Object(hm);
        let label = out.extract_output(&element, "my_config");
        assert!(label.is_some());
        let output = out.output;
        assert!(output.contains("max_items: i32,"));
        assert!(
            output.contains("/// FIXME: Describe the purpose of this field."),
            "output did not contain doc comment: {output}"
        );
    }

    #[test]
    fn test_parsing_more_configs() {
        let mut out = RuleConfigOutput::new(false);
        let mut hm = FxHashMap::default();
        hm.insert("enabled".to_string(), (RuleConfigElement::Boolean, None));
        hm.insert("threshold".to_string(), (RuleConfigElement::Number, None));
        let element = RuleConfigElement::Object(hm);
        let label = out.extract_output(&element, "another_config");
        assert!(label.is_some());
        let output = out.output;
        assert!(output.contains("enabled: bool,"));
        assert!(output.contains("threshold: f32,"));
        assert!(
            output.contains("/// FIXME: Describe the purpose of this field."),
            "output did not contain doc comment: {output}"
        );
    }

    #[test]
    fn test_struct_with_reserved_name_in_fields() {
        let mut out = RuleConfigOutput::new(false);
        let mut hm = FxHashMap::default();
        hm.insert("type".to_string(), (RuleConfigElement::String, None));
        hm.insert("match".to_string(), (RuleConfigElement::String, None));
        hm.insert("fn".to_string(), (RuleConfigElement::String, None));
        let element = RuleConfigElement::Object(hm);
        let label = out.extract_output(&element, "reserved_names_config");
        assert!(label.is_some());
        let output = out.output;
        assert!(output.contains("r#type: String,"));
        assert!(output.contains("r#match: String,"));
        assert!(output.contains("r#fn: String,"));
        assert!(
            output.contains("/// FIXME: Describe the purpose of this field."),
            "output did not contain doc comment: {output}"
        );
    }

    #[test]
    fn test_enum_rename_all_kebab() {
        let mut out = RuleConfigOutput::new(false);
        let element = RuleConfigElement::Enum(vec![
            RuleConfigElement::StringLiteral("type-based".to_string()),
            RuleConfigElement::StringLiteral("type-literal".to_string()),
        ]);
        let label = out.extract_output(&element, "define_emits_declaration_config");
        assert!(label.is_some());
        let output = out.output;
        assert!(output.contains("rename_all = \"kebab-case\""));
        assert!(!output.contains("rename = \""));
    }

    #[test]
    fn test_enum_mixed_case_keeps_per_variant() {
        let mut out = RuleConfigOutput::new(false);
        let element = RuleConfigElement::Enum(vec![
            RuleConfigElement::StringLiteral("beforeAll".to_string()),
            RuleConfigElement::StringLiteral("after-each".to_string()),
        ]);
        let label = out.extract_output(&element, "no_hooks_config");
        assert!(label.is_some());
        let output = out.output;
        // mixed case shouldn't set kebab-case rename_all
        assert!(!output.contains("rename_all = \"kebab-case\""));
        // but should have explicit per-variant rename for kebab one
        assert!(output.contains("rename = \"after-each\""));
    }

    #[test]
    fn test_enum_with_non_string_includes_untagged() {
        let mut out = RuleConfigOutput::new(false);
        let mut hm = FxHashMap::default();
        hm.insert("a".to_string(), (RuleConfigElement::String, None));
        let element = RuleConfigElement::Enum(vec![
            RuleConfigElement::StringLiteral("foo".to_string()),
            RuleConfigElement::Object(hm),
        ]);
        let label = out.extract_output(&element, "mixed_config");
        assert!(label.is_some());
        let output = out.output;
        // non-string variants should cause `untagged` to be present in the serde attribute.
        assert!(output.contains("untagged"), "output did not contain untagged: {output}");
    }

    #[test]
    fn test_enum_with_strings_does_not_include_untagged() {
        let mut out = RuleConfigOutput::new(false);
        let element = RuleConfigElement::Enum(vec![
            RuleConfigElement::StringLiteral("foo".to_string()),
            RuleConfigElement::StringLiteral("bar".to_string()),
        ]);
        let label = out.extract_output(&element, "string_only_config");
        assert!(label.is_some());
        let output = out.output;
        // all-string variants should NOT cause `untagged` to be present in the serde attribute.
        assert!(!output.contains("untagged"), "output contained untagged: {output}");
    }

    #[test]
    fn test_struct_with_defaults() {
        let mut out = RuleConfigOutput::new(false);
        let mut hm = FxHashMap::default();
        hm.insert("maxItems".to_string(), (RuleConfigElement::Integer, Some("2".to_string())));
        let element = RuleConfigElement::Object(hm);
        let label = out.extract_output(&element, "my_config");
        assert!(label.is_some());
        let output = out.output;
        // match serde(rename_all = "camelCase", default)""
        assert!(output.contains("#[serde(rename_all = \"camelCase\", default)]"));
        assert!(output.contains("impl Default for MyConfig"));
        assert!(output.contains("max_items: 2,"));
        assert!(!output.contains("derive(Debug, Default"));
        assert!(
            output.contains("/// FIXME: Describe the purpose of this field."),
            "output did not contain doc comment: {output}"
        );
    }

    #[test]
    fn test_struct_default_literal_types() {
        // string default
        let mut out = RuleConfigOutput::new(false);
        let mut hm = FxHashMap::default();
        hm.insert("name".to_string(), (RuleConfigElement::String, Some(r#""bar""#.to_string())));
        let element = RuleConfigElement::Object(hm);
        let label = out.extract_output(&element, "my_config");
        assert!(label.is_some());
        let output = out.output;
        assert!(output.contains("name: String::from(\"bar\"),"));
        assert!(
            output.contains("/// FIXME: Describe the purpose of this field."),
            "output did not contain doc comment: {output}"
        );

        // boolean default
        let mut out = RuleConfigOutput::new(false);
        let mut hm = FxHashMap::default();
        hm.insert("enabled".to_string(), (RuleConfigElement::Boolean, Some("true".to_string())));
        let element = RuleConfigElement::Object(hm);
        let label = out.extract_output(&element, "my_config");
        assert!(label.is_some());
        let output = out.output;
        assert!(output.contains("enabled: true,"));

        // float/default number
        let mut out = RuleConfigOutput::new(false);
        let mut hm = FxHashMap::default();
        hm.insert("threshold".to_string(), (RuleConfigElement::Number, Some("1.5".to_string())));
        let element = RuleConfigElement::Object(hm);
        let label = out.extract_output(&element, "my_config");
        assert!(label.is_some());
        let output = out.output;
        assert!(output.contains("threshold: 1.5f32,"));

        // Option null default -> None
        let mut out = RuleConfigOutput::new(false);
        let mut hm = FxHashMap::default();
        hm.insert(
            "maybeValue".to_string(),
            (
                RuleConfigElement::Nullable(Box::new(RuleConfigElement::Integer)),
                Some("null".to_string()),
            ),
        );
        let element = RuleConfigElement::Object(hm);
        let label = out.extract_output(&element, "my_config");
        assert!(label.is_some());
        let output = out.output;
        assert!(output.contains("maybe_value: None,"));

        // Option string default -> Some(String::from("x"))
        let mut out = RuleConfigOutput::new(false);
        let mut hm = FxHashMap::default();
        hm.insert(
            "s".to_string(),
            (
                RuleConfigElement::Nullable(Box::new(RuleConfigElement::String)),
                Some(r#""x""#.to_string()),
            ),
        );
        let element = RuleConfigElement::Object(hm);
        let label = out.extract_output(&element, "my_config");
        assert!(label.is_some());
        let output = out.output;
        assert!(output.contains("s: Some(String::from(\"x\")),"));
    }

    #[test]
    fn test_struct_default_array_and_object_unsupported() {
        // array default (supported)
        let mut out = RuleConfigOutput::new(true);
        let mut hm = FxHashMap::default();
        hm.insert(
            "items".to_string(),
            (
                RuleConfigElement::Array(Box::new(RuleConfigElement::Integer)),
                Some("[1, 2]".to_string()),
            ),
        );
        let element = RuleConfigElement::Object(hm);
        let label = out.extract_output(&element, "my_config");
        assert!(label.is_some());
        let output = out.output;
        // Array default should render to vec![...] and not error
        assert!(output.contains("items: vec![1, 2],"));
        assert!(!out.has_errors, "did not expect errors for supported array default");

        // array of strings default (supported)
        let mut out_str = RuleConfigOutput::new(true);
        let mut hm_str = FxHashMap::default();
        hm_str.insert(
            "tags".to_string(),
            (
                RuleConfigElement::Array(Box::new(RuleConfigElement::String)),
                Some("['a', 'b']".to_string()),
            ),
        );
        let element_str = RuleConfigElement::Object(hm_str);
        let label_str = out_str.extract_output(&element_str, "my_config");
        assert!(label_str.is_some());
        let output_str = out_str.output;
        assert!(output_str.contains("tags: vec![String::from(\"a\"), String::from(\"b\")],"));
        assert!(!out_str.has_errors, "did not expect errors for supported string array default");

        // object default (unsupported)
        let mut out2 = RuleConfigOutput::new(true);
        let mut hm2 = FxHashMap::default();
        hm2.insert(
            "map".to_string(),
            (
                RuleConfigElement::Map(Box::new(RuleConfigElement::String)),
                Some("{ \"a\": 1 }".to_string()),
            ),
        );
        let element2 = RuleConfigElement::Object(hm2);
        let label2 = out2.extract_output(&element2, "my_config2");
        assert!(label2.is_some());
        let output2 = out2.output;
        assert!(output2.contains("map: Default::default(),"));
        assert!(out2.has_errors, "expected error logged for unsupported default on object");
    }

    #[test]
    fn test_parsing_default_from_rule_file() {
        let source = r#"
            export default {
                meta: {
                    schema: {
                        type: "object",
                        properties: {
                            name: { type: "string", default: "bar" },
                            count: { type: "integer", default: 5 }
                        }
                    }
                }
            };
        "#;

        let allocator = Allocator::default();
        let ret =
            Parser::new(&allocator, source, SourceType::from_path("rule.js").unwrap()).parse();
        assert!(ret.errors.is_empty(), "parse errors: {:?}", ret.errors);

        let mut rc = RuleConfig::new(source, false);
        rc.visit_program(&ret.program);
        let element = rc
            .next_element
            .as_ref()
            .or_else(|| rc.elements.first())
            .expect("expected schema element");

        let mut out = RuleConfigOutput::new(false);
        let label = out.extract_output(element, "my_config");
        assert!(label.is_some());
        let output = out.output;
        assert!(output.contains("name: String::from(\"bar\"),"));
        assert!(output.contains("count: 5,"));
        assert!(output.contains("impl Default for MyConfig"));
    }

    #[test]
    fn test_format_code_snippet_simple() {
        assert_eq!(format_code_snippet("debugger"), "\"debugger\"");
    }

    #[test]
    fn test_format_code_snippet_with_quotes() {
        let code = r#"import foo from "foo";"#;
        let expected = format!("r#\"{code}\"#");
        assert_eq!(format_code_snippet(code), expected);
    }

    #[test]
    fn test_format_code_snippet_with_hash_in_quote() {
        let code = r##"document.querySelector("#foo");"##;
        let expected = format!("r##\"{code}\"##");
        assert_eq!(format_code_snippet(code), expected);
    }

    #[test]
    fn test_format_code_snippet_raw_preserved() {
        let raw = r#"r"foo""#;
        assert_eq!(format_code_snippet(raw), raw);
    }

    #[test]
    fn test_format_code_snippet_with_backslash() {
        let code = "\\u1234";
        let expected = format!("r#\"{code}\"#");
        assert_eq!(format_code_snippet(code), expected);
    }

    #[test]
    fn test_format_code_snippet_multiline() {
        let code = "a\nb";
        let expected = "\"a\n\t\t\tb\"";
        assert_eq!(format_code_snippet(code), expected);
    }

    /// Generate the rendered template for a rule using test and source fixtures.
    fn generate_rule_template_from_fixtures(
        rule_kind: RuleKind,
        kebab_rule_name: &str,
        test_path: &str,
        src_path: &str,
        language: &str,
        pascal_rule_name: &str,
    ) -> String {
        // Load and parse test fixtures
        let test_body = std::fs::read_to_string(test_path).expect("fixture test file");
        let parsed = parse_test_file(&test_body, test_path).expect("failed to parse test file");

        // Build context from parsed test
        let mut ctx = build_context_from_parsed_test(parsed, rule_kind, kebab_rule_name, language);

        // Load and parse rule source for config
        let src_body = std::fs::read_to_string(src_path).expect("fixture rule source file");
        let elements =
            parse_rule_source(&src_body, src_path, false).expect("failed to parse rule source");

        // Apply rule config to context
        ctx = apply_rule_config_to_context(ctx, &elements, pascal_rule_name, false);

        // Render template and return
        let mut registry = handlebars::Handlebars::new();
        registry.register_escape_fn(handlebars::no_escape);
        registry
            .render_template(include_str!("../template.txt"), &handlebars::to_json(&ctx))
            .expect("Failed to render template")
    }

    #[test]
    fn test_jest_no_large_snapshots_rulegen() {
        let rendered = generate_rule_template_from_fixtures(
            RuleKind::Jest,
            "no_large_snapshots",
            "tests/fixtures/jest/__tests__/no-large-snapshots.test.ts",
            "tests/fixtures/jest/no-large-snapshots.ts",
            "ts",
            "NoLargeSnapshots",
        );
        insta::assert_snapshot!("rulegen_jest_no_large_snapshots", rendered);
    }

    #[test]
    fn test_vue_define_emits_declaration_rulegen() {
        let rendered = generate_rule_template_from_fixtures(
            RuleKind::Vue,
            "define-emits-declaration",
            "tests/fixtures/vue/__tests__/define-emits-declaration.test.js",
            "tests/fixtures/vue/define-emits-declaration.js",
            "js",
            "DefineEmitsDeclaration",
        );
        insta::assert_snapshot!("rulegen_vue_define_emits_declaration", rendered);
    }
}
