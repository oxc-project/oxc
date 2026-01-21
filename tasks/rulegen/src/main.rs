#![expect(clippy::print_stdout, clippy::print_stderr, clippy::disallowed_methods)]
use std::fmt::Write as _;
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    fs,
    path::Path,
};

use convert_case::{Case, Casing};
use lazy_regex::regex;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, AssignmentTarget, CallExpression,
    Expression, ExpressionStatement, IdentifierName, ObjectExpression, ObjectProperty,
    ObjectPropertyKind, Program, PropertyKey, Statement, StaticMemberExpression, StringLiteral,
    TaggedTemplateExpression, TemplateLiteral,
};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use oxc_tasks_common::project_root;
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

    fn output(&self, need_config: bool) -> Option<String> {
        let code = format_code_snippet(self.code.as_ref()?);
        let output = format_code_snippet(self.output.as_ref()?);

        if need_config {
            let config = self.config.as_ref().map_or_else(
                || "None".to_string(),
                |config| format!("Some(serde_json::json!({config}))"),
            );
            Some(format!(r"({code}, {output}, {config})"))
        } else {
            Some(format!(r"({code}, {output})"))
        }
    }
}

fn format_code_snippet(code: &str) -> String {
    let code = if code.contains('\n') {
        // Use 12 space characters after the newline.
        code.replace('\n', "\n            ").replace('\\', "\\\\").replace('\"', "\\\"")
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
    if code.contains('"') {
        return format!("r#\"{}\"#", code.replace("\\\"", "\""));
    }

    // `foo === bar` => `r"foo === bar"`
    format!("r\"{code}\"")
}
// TODO: handle `noFormat`(in typescript-eslint)
fn format_tagged_template_expression(tag_expr: &TaggedTemplateExpression) -> Option<String> {
    // Some test cases use code like this (e.g. no-invalid-regex):
    // ```js
    // String.raw`foobar\n\t escapedthing`
    // ```
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
    Object(FxHashMap<String, RuleConfigElement>),
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
                output.push_str("#[serde(untagged, rename_all = \"camelCase\")]\n");
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
                                let rename = if formatted_string_literal.to_case(Case::Camel)
                                    == *string_literal
                                {
                                    None
                                } else {
                                    Some(format!("rename = \"{string_literal}\""))
                                };
                                Some((rename, Some(formatted_string_literal), None, None))
                            } else {
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
                let mut output = String::from(
                    "#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]\n",
                );
                output.push_str(
                    "#[serde(rename_all = \"camelCase\", default, deny_unknown_fields)]\n",
                );
                let _ = writeln!(output, "struct {struct_name} {{");
                let mut fields_output = String::new();
                for (raw_key, value) in hash_map {
                    let key = &raw_key.to_case(Case::Pascal);
                    let Some((value_label, value_output)) = self.extract_output_inner(value, key)
                    else {
                        continue;
                    };
                    if key.to_case(Case::Camel) != *raw_key {
                        let _ = writeln!(output, "    #[serde(rename = \"{raw_key}\")]");
                    }
                    let _ = writeln!(output, "    {}: {value_label},", key.to_case(Case::Snake));
                    if let Some(value_output) = value_output {
                        let _ = writeln!(fields_output, "{value_output}\n");
                    }
                }
                let _ = writeln!(output, "}}\n{fields_output}");
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
}

struct RuleConfig<'a> {
    elements: Vec<RuleConfigElement>,
    next_element: Option<RuleConfigElement>,
    source_text: &'a str,
    has_errors: bool,
    log_errors: bool,
}

impl<'a> RuleConfig<'a> {
    fn new(source_text: &'a str, log_errors: bool) -> Self {
        Self { elements: vec![], next_element: None, source_text, has_errors: false, log_errors }
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
            "array" | "object" => None,
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
    ) -> FxHashMap<String, RuleConfigElement> {
        let mut properties: FxHashMap<String, RuleConfigElement> = FxHashMap::default();
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
            self.visit_object_expression(object_expression);
            let Some(element) = self.next_element.take() else {
                self.log_error(&String::from("Cannot find next element"));
                continue;
            };
            properties.insert(identifier.name.into(), element);
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
                "default" | "required" | "minItems" | "minimum" | "minLength" | "maxItems"
                | "minProperties" | "maximum" | "pattern" => {}
                _ => {
                    self.log_error(&format!("Unhandled key `{}`", identifier.name));
                }
            }
        }
        self.next_element = rule_config_element;
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

    let update_tests_only = std::env::args().any(|arg| arg == "--update-tests");
    let kebab_rule_name = rule_name.to_case(Case::Kebab);
    let camel_rule_name = rule_name.to_case(Case::Camel);

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

    // Check whether this rule is marked as unsupported, and bail if it is.
    if let Some((key, reason)) = find_unsupported_rule(&kebab_rule_name, rule_kind) {
        eprintln!();
        eprintln!("Error: This rule is either not planned or not possible to implement in Oxlint.");
        eprintln!("The rule '{key}' will not be implemented for the following reason:");
        eprintln!("> {reason}");
        eprintln!();
        std::process::exit(1);
    }

    println!("Reading test file from {rule_test_path}");

    let test_body = oxc_tasks_common::agent()
        .get(&rule_test_path)
        .call()
        .map(|mut res| res.body_mut().read_to_string());
    let mut context = match test_body {
        Ok(Ok(body)) => {
            let allocator = Allocator::default();
            let source_type = SourceType::from_path(rule_test_path).unwrap();
            let ret = Parser::new(&allocator, &body, source_type).parse();
            assert!(ret.errors.is_empty());

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
                // Check if any case with output has a config
                let need_config =
                    cases.iter().any(|case| case.output.is_some() && case.config.is_some());
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

                    if let Some(output) = case.output(need_config) {
                        fix_codes.push(output);
                    }

                    codes.push(code);
                }

                (codes.join(",\n"), fix_codes.join(",\n"))
            };

            // pass cases don't need to be fixed
            let (pass_cases, _) = gen_cases_string(pass_cases);
            let (fail_cases, fix_cases) = gen_cases_string(fail_cases);

            Context::new(rule_kind, &rule_name, pass_cases, fail_cases)
                .with_language(language)
                .with_filename(has_filename)
                .with_fix_cases(fix_cases)
        }
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
            let allocator = Allocator::default();
            let source_type = SourceType::from_path(rule_src_path).unwrap();
            let ret = Parser::new(&allocator, &body, source_type).parse();
            assert!(ret.errors.is_empty());
            let debug_mode = false;
            let mut config = RuleConfig::new(&body, debug_mode);
            // TODO: Use the tasks/lint_rules package to get the runtime config object from javascript
            // and parse it here to resolve values of expressions.
            config.visit_program(&ret.program);
            if debug_mode {
                println!("Rule config: {:?}", config.elements);
            }
            let mut rule_config_output = RuleConfigOutput::new(debug_mode);
            let config_names = config
                .elements
                .iter()
                .enumerate()
                .filter_map(|(index, element)| {
                    let element_name = format!("ConfigElement{index}");
                    rule_config_output.extract_output(element, element_name.as_str())
                })
                .collect::<Vec<_>>();
            if debug_mode {
                println!("Rule config names: {config_names:?}");
                println!("Rule Output:\n{}", rule_config_output.output);
            }
            if rule_config_output.has_errors {
                println!("Rule config parsed, but with fatal errors. Not writing config.");
            } else if config.has_errors {
                println!("Rule config parsed, but with errors.");
            } else {
                println!("Rule config parsed.");
            }
            if !config_names.is_empty() && !rule_config_output.has_errors {
                let rule_config_tuple = format!("({})", config_names.join(", "));
                context = context.with_rule_config(
                    rule_config_output.output,
                    rule_config_tuple,
                    rule_config_output.has_hash_map,
                    rule_config_output.has_hash_set,
                );
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

    if update_tests_only {
        if let Err(err) = update_test_block(&context, rule_kind) {
            eprintln!("failed to update test block for {rule_name}: {err}");
        }
    } else {
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

    Ok(())
}

/// Updates only the test block in an existing rule file, preserving the rule implementation.
fn update_test_block(ctx: &Context, rule_kind: RuleKind) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    let path = get_rule_path(rule_kind);
    let rule_file_path = path.join(format!("{}.rs", ctx.snake_rule_name));

    if !rule_file_path.exists() {
        return Err(format!(
            "Rule file '{}' does not exist. Use rulegen without --update-tests to create it first.",
            rule_file_path.display()
        )
        .into());
    }

    let existing_content = std::fs::read_to_string(&rule_file_path)?;

    // Find the test function in the existing file using syn parser
    let (test_start, test_end) = find_test_function_range(&existing_content)
        .ok_or("Could not find '#[test] fn test()' in the existing rule file")?;

    // Generate the new test block
    let new_test_block = generate_test_block(ctx);

    // Replace the old test block with the new one
    let mut new_content = String::with_capacity(existing_content.len());
    new_content.push_str(&existing_content[..test_start]);
    new_content.push_str(&new_test_block);
    new_content.push_str(&existing_content[test_end..]);

    std::fs::write(&rule_file_path, new_content)?;
    println!("Updated test block in {}", rule_file_path.display());

    // Format the file
    let res = Command::new("cargo")
        .arg("fmt")
        .arg("--")
        .arg(&rule_file_path)
        .spawn()
        .map(|mut child| child.wait().expect("failed to format"));

    match res {
        Ok(exit_status) if exit_status.success() => println!("Formatted rule file"),
        Ok(exit_status) => println!("Failed to format rule file: exited with {exit_status}"),
        Err(e) => println!("Failed to format rule file: {e}"),
    }

    Ok(())
}

/// Returns the path to the rules directory for a given rule kind.
fn get_rule_path(rule_kind: RuleKind) -> &'static Path {
    match rule_kind {
        RuleKind::ESLint => Path::new("crates/oxc_linter/src/rules/eslint"),
        RuleKind::Jest => Path::new("crates/oxc_linter/src/rules/jest"),
        RuleKind::Typescript => Path::new("crates/oxc_linter/src/rules/typescript"),
        RuleKind::Unicorn => Path::new("crates/oxc_linter/src/rules/unicorn"),
        RuleKind::Import => Path::new("crates/oxc_linter/src/rules/import"),
        RuleKind::React => Path::new("crates/oxc_linter/src/rules/react"),
        RuleKind::ReactPerf => Path::new("crates/oxc_linter/src/rules/react_perf"),
        RuleKind::JSXA11y => Path::new("crates/oxc_linter/src/rules/jsx_a11y"),
        RuleKind::Oxc => Path::new("crates/oxc_linter/src/rules/oxc"),
        RuleKind::NextJS => Path::new("crates/oxc_linter/src/rules/nextjs"),
        RuleKind::JSDoc => Path::new("crates/oxc_linter/src/rules/jsdoc"),
        RuleKind::Node => Path::new("crates/oxc_linter/src/rules/node"),
        RuleKind::Promise => Path::new("crates/oxc_linter/src/rules/promise"),
        RuleKind::Vitest => Path::new("crates/oxc_linter/src/rules/vitest"),
        RuleKind::Vue => Path::new("crates/oxc_linter/src/rules/vue"),
    }
}

/// Finds the byte range (start, end) of the `#[test] fn test()` function in the content.
/// Uses `syn` to properly parse the Rust source code.
fn find_test_function_range(content: &str) -> Option<(usize, usize)> {
    use syn::spanned::Spanned;

    let file = syn::parse_file(content).ok()?;

    for item in &file.items {
        if let syn::Item::Fn(item_fn) = item {
            // Check if function is named "test" and has #[test] attribute
            if item_fn.sig.ident == "test" {
                let has_test_attr = item_fn.attrs.iter().any(|attr| attr.path().is_ident("test"));
                if has_test_attr {
                    let span = item_fn.span();
                    let start = span.start();
                    let end = span.end();

                    // Convert line/column to byte offset
                    let start_offset = line_col_to_offset(content, start.line, start.column)?;
                    let end_offset = line_col_to_offset(content, end.line, end.column)?;

                    return Some((start_offset, end_offset));
                }
            }
        }
    }

    None
}

/// Converts a 1-based line number and 0-based column to a byte offset.
fn line_col_to_offset(content: &str, line: usize, column: usize) -> Option<usize> {
    let mut current_line = 1;
    let mut line_start = 0;

    for (i, c) in content.char_indices() {
        if current_line == line {
            // Found the line, now advance by column (in bytes for the chars)
            let line_content = &content[line_start..];
            let col_offset: usize = line_content.chars().take(column).map(char::len_utf8).sum();
            return Some(line_start + col_offset);
        }
        if c == '\n' {
            current_line += 1;
            line_start = i + 1;
        }
    }

    // Handle last line
    if current_line == line {
        let line_content = &content[line_start..];
        let col_offset: usize = line_content.chars().take(column).map(char::len_utf8).sum();
        return Some(line_start + col_offset);
    }

    None
}

/// Generates the test block content from the context.
fn generate_test_block(ctx: &Context) -> String {
    let mut test_block = String::new();

    test_block.push_str("#[test]\nfn test() {\n");
    test_block.push_str("    use crate::tester::Tester;\n");

    if ctx.has_filename {
        test_block.push_str("    use std::path::PathBuf;\n");
    }

    test_block.push_str("\n    let pass = vec![\n");
    if !ctx.pass_cases.is_empty() {
        test_block.push_str("        ");
        test_block.push_str(&ctx.pass_cases);
        test_block.push('\n');
    }
    test_block.push_str("    ];\n");

    test_block.push_str("\n    let fail = vec![\n");
    if !ctx.fail_cases.is_empty() {
        test_block.push_str("        ");
        test_block.push_str(&ctx.fail_cases);
        test_block.push('\n');
    }
    test_block.push_str("    ];\n\n");

    if let Some(fix_cases) = &ctx.fix_cases
        && !fix_cases.is_empty()
    {
        test_block.push_str("    let fix = vec![\n");
        test_block.push_str("        ");
        test_block.push_str(fix_cases);
        test_block.push_str("\n    ];\n\n");
        let _ = writeln!(
            test_block,
            "    Tester::new({}::NAME, {}::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();",
            ctx.pascal_rule_name, ctx.pascal_rule_name
        );
    } else {
        let _ = writeln!(
            test_block,
            "    Tester::new({}::NAME, {}::PLUGIN, pass, fail).test_and_snapshot();",
            ctx.pascal_rule_name, ctx.pascal_rule_name
        );
    }

    test_block.push_str("}\n");

    test_block
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

/// Returns the plugin prefix used in `unsupported-rules.json` for a given `RuleKind`.
fn get_unsupported_rule_prefix(rule_kind: RuleKind) -> &'static str {
    match rule_kind {
        RuleKind::ESLint => "eslint",
        RuleKind::Jest => "jest",
        RuleKind::Typescript => "typescript",
        RuleKind::Unicorn => "unicorn",
        RuleKind::Import => "import",
        RuleKind::React => "react",
        RuleKind::ReactPerf => "react-perf",
        RuleKind::JSXA11y => "jsx-a11y",
        RuleKind::Oxc => "oxc",
        RuleKind::NextJS => "nextjs",
        RuleKind::JSDoc => "jsdoc",
        RuleKind::Node => "n",
        RuleKind::Promise => "promise",
        RuleKind::Vitest => "vitest",
        RuleKind::Vue => "vue",
    }
}

/// This returns the rule key if it's listed as unsupported (meaning
/// it isn't possible or isn't planned for support), otherwise it
/// returns nothing.
///
/// See `tasks/lint_rules/src/unsupported-rules.json` for reference.
fn find_unsupported_rule(kebab_rule: &str, rule_kind: RuleKind) -> Option<(String, String)> {
    let file_path = project_root().join("tasks/lint_rules/src/unsupported-rules.json");
    let json = fs::read_to_string(file_path)
        .expect("Failed to read file")
        .parse::<serde_json::Value>()
        .expect("Failed to parse JSON");

    let map = json.get("unsupportedRules")?.as_object()?;

    let prefix = get_unsupported_rule_prefix(rule_kind);
    let full_rule_key = format!("{prefix}/{kebab_rule}");

    // Check for exact match with the full rule key (e.g., "eslint/dot-notation")
    if let Some(val) = map.get(&full_rule_key) {
        let reason = val.as_str().unwrap_or("").to_string();
        return Some((full_rule_key, reason));
    }

    None
}

/// Adds a module definition for the given rule to the `rules.rs` file.
/// The `RuleEnum` will be regenerated by `oxc_linter_codegen` which is called after this function.
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

    if mod_rules.contains(&rule_mod_def) {
        println!("Rule module '{}' already exists in {rules_path}", ctx.snake_rule_name);
        return Ok(());
    }

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

    std::fs::write(rules_path, rules)?;
    println!("Updated {rules_path}");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_unsupported_rule_matches() {
        // `eslint/no-dupe-args` should be detected by the helper as it's an unsupported rule.
        let res = find_unsupported_rule("no-dupe-args", RuleKind::ESLint);
        assert!(res.is_some());
        let (key, reason) = res.unwrap();
        assert!(key == "eslint/no-dupe-args");
        assert!(!reason.is_empty());
    }

    #[test]
    fn test_find_unsupported_rule_does_not_match() {
        // This rule is not in the unsupported list, so it should return None.
        let res = find_unsupported_rule("foobar", RuleKind::ESLint);
        assert!(res.is_none());
    }

    #[test]
    fn test_find_unsupported_rule_matches_for_n_rule() {
        let res = find_unsupported_rule("no-hide-core-modules", RuleKind::Node);
        assert!(res.is_some());
        let (key, reason) = res.unwrap();
        assert!(key == "n/no-hide-core-modules");
        assert!(!reason.is_empty());
    }

    #[test]
    fn test_format_code_snippet_simple_identifier() {
        assert_eq!(format_code_snippet("debugger"), "\"debugger\"");
    }

    #[test]
    fn test_format_code_snippet_simple_expression() {
        // Simple expression uses regular quotes
        assert_eq!(format_code_snippet("foo === bar"), "\"foo === bar\"");
    }

    #[test]
    fn test_format_code_snippet_with_double_quotes() {
        // Code containing double quotes uses raw string
        assert_eq!(
            format_code_snippet("import foo from \"foo\";"),
            "r#\"import foo from \"foo\";\"#"
        );
    }

    #[test]
    fn test_format_code_snippet_with_hash_in_quotes() {
        // Code containing "#" with a double-quote, needs raw string with `r##`.
        assert_eq!(
            format_code_snippet("document.querySelector(\"#foo\");"),
            "r##\"document.querySelector(\"#foo\");\"##"
        );
    }

    #[test]
    fn test_format_code_snippet_multiline() {
        // Multiline code with newlines gets escaped and indented
        let input = "const x = 1;\nconst y = 2;";
        let result = format_code_snippet(input);
        assert!(result.contains("const x = 1;"));
        assert!(result.contains("const y = 2;"));
    }

    #[test]
    fn test_format_code_snippet_with_backslash() {
        // Code containing backslashes uses raw string
        let input = "\\u1234";
        let result = format_code_snippet(input);
        assert!(result.starts_with("r\""));
    }

    #[test]
    fn test_format_code_snippet_raw_string_r_quote() {
        // Already a raw string with r" - returned as-is
        assert_eq!(format_code_snippet("r\"foobar\""), "r\"foobar\"");
    }

    #[test]
    fn test_format_code_snippet_raw_string_r_hash_quote() {
        // Already a raw string with r#" - returned as-is
        assert_eq!(format_code_snippet("r#\"foobar\"#"), "r#\"foobar\"#");
    }

    #[test]
    fn test_format_code_snippet_quoted_string() {
        // Quoted string as code - contains both quotes and no backslashes
        let result = format_code_snippet("\"debugger\"");
        assert!(result.starts_with("r#"));
        assert!(result.contains("debugger"));
    }

    #[test]
    fn test_format_code_snippet_complex_multiline() {
        // Complex multiline with multiple special characters
        let input = "function test() {\n  console.log(\"hello\");\n}";
        let result = format_code_snippet(input);
        assert!(result.contains("function test()"));
        assert!(result.contains("console.log("));
    }

    #[test]
    fn test_format_code_snippet_empty_string() {
        // Empty string code
        assert_eq!(format_code_snippet(""), "\"\"");
    }

    #[test]
    fn test_format_code_snippet_with_tab_characters() {
        // Code containing tab characters (no quotes or backslashes)
        assert_eq!(
            format_code_snippet("const x = 1;\tconst y = 2;"),
            "\"const x = 1;\tconst y = 2;\""
        );
    }

    #[test]
    fn test_format_code_snippet_multiline_with_quotes() {
        // Multiline code with double quotes
        let input = "const msg = \"hello\";\nconsole.log(msg);";
        let result = format_code_snippet(input);
        assert!(result.contains("const msg = "));
        assert!(result.contains("console.log(msg);"));
        assert!(result.starts_with("r#\""));
    }

    #[test]
    fn test_format_code_snippet_regex_pattern() {
        // Regex pattern with backslashes
        let input = "/\\d+/";
        let result = format_code_snippet(input);
        assert!(result.starts_with("r\""));
    }

    #[test]
    fn test_format_code_snippet_unicode_escape() {
        // Unicode escape sequences - has backslash so uses raw string
        let result = format_code_snippet("\\u0041");
        assert!(result.starts_with("r\""));
        assert!(result.contains("u0041"));
    }

    #[test]
    fn test_format_code_snippet_mixed_quotes_and_hash() {
        // Both quotes and # present - needs r##
        let input = "document.getElementById(\"#id\")";
        let result = format_code_snippet(input);
        assert!(result.starts_with("r##"));
    }

    #[test]
    fn test_format_code_snippet_newline_normalization() {
        // Newline characters get indented with spaces, not tabs
        let input = "line1\nline2\nline3";
        let result = format_code_snippet(input);
        assert!(result.contains("line1"));
        assert!(result.contains("line2"));
        assert!(result.contains("line3"));
        assert!(!result.contains('\t'));
    }

    #[test]
    fn test_format_code_snippet_double_backslash() {
        // Code with escaped backslash
        let input = "path\\\\to\\\\file";
        let result = format_code_snippet(input);
        assert!(result.starts_with("r\""));
    }

    #[test]
    fn test_format_code_snippet_single_quotes() {
        // Single quotes (no double quotes) - treated as regular string
        assert_eq!(format_code_snippet("const msg = 'hello';"), "\"const msg = 'hello';\"");
    }
}
