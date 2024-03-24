use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use convert_case::{Case, Casing};
use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, CallExpression, ExportDefaultDeclarationKind, Expression,
        ExpressionStatement, MemberExpression, ModuleDeclaration, ObjectExpression, ObjectProperty,
        ObjectPropertyKind, Program, PropertyKey, Statement, StaticMemberExpression, StringLiteral,
        TaggedTemplateExpression, TemplateLiteral,
    },
    Visit,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use serde::Serialize;
use ureq::Response;

mod json;
mod template;

const ESLINT_TEST_PATH: &str =
    "https://raw.githubusercontent.com/eslint/eslint/main/tests/lib/rules";

const JEST_TEST_PATH: &str =
    "https://raw.githubusercontent.com/jest-community/eslint-plugin-jest/main/src/rules/__tests__";

const TYPESCRIPT_ESLINT_TEST_PATH: &str =
    "https://raw.githubusercontent.com/typescript-eslint/typescript-eslint/main/packages/eslint-plugin/tests/rules";

const UNICORN_TEST_PATH: &str =
    "https://raw.githubusercontent.com/sindresorhus/eslint-plugin-unicorn/main/test";

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

struct TestCase<'a> {
    source_text: String,
    code: Option<String>,
    group_comment: Option<String>,
    config: Option<Cow<'a, str>>,
    settings: Option<Cow<'a, str>>,
}

impl<'a> TestCase<'a> {
    fn new(source_text: &str, arg: &'a Expression<'a>) -> Self {
        let mut test_case = Self {
            source_text: source_text.to_string(),
            code: None,
            config: None,
            settings: None,
            group_comment: None,
        };
        test_case.visit_expression(arg);
        test_case
    }

    fn with_group_comment(mut self, comment: String) -> Self {
        self.group_comment = Some(comment);
        self
    }

    fn code(&self, need_config: bool, need_settings: bool) -> String {
        self.code
            .as_ref()
            .map(|code| {
                let mut code = if code.contains('\n') {
                    code.replace('\n', "\n\t\t\t").replace('\\', "\\\\").replace('\"', "\\\"")
                } else {
                    code.to_string()
                };

                if code.contains('"') {
                    // handle " to \" and then \\" to \"
                    code = code.replace('"', "\\\"").replace("\\\\\"", "\\\"");
                }

                let config = self.config.as_ref().map_or_else(
                    || "None".to_string(),
                    |config| format!("Some(serde_json::json!({config}))"),
                );
                let settings = self.settings.as_ref().map_or_else(
                    || "None".to_string(),
                    |settings| format!("Some(serde_json::json!({settings}))"),
                );
                let code_str = if code.contains('"') {
                    format!("r#\"{}\"#", code.replace("\\\"", "\""))
                } else {
                    format!("\"{code}\"")
                };
                if need_settings {
                    format!("({code_str}, {config}, {settings})")
                } else if need_config {
                    format!("({code_str}, {config})")
                } else {
                    code_str.to_string()
                }
            })
            .unwrap_or_default()
    }

    fn group_comment(&self) -> Option<&str> {
        self.group_comment.as_deref()
    }
}

impl<'a> Visit<'a> for TestCase<'a> {
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
        if let Expression::MemberExpression(member_expr) = &expr.callee {
            if let Expression::ArrayExpression(array_expr) = member_expr.object() {
                // ['class A {', '}'].join('\n')
                let mut code = String::new();
                for arg in &array_expr.elements {
                    let ArrayExpressionElement::Expression(Expression::StringLiteral(lit)) = arg
                    else {
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
                    PropertyKey::Identifier(ident) if ident.name == "code" => {
                        self.code = match &prop.value {
                            Expression::StringLiteral(s) => Some(s.value.to_string()),
                            // eslint-plugin-jest use dedent to strips indentation from multi-line strings
                            // eslint-plugin-unicon use outdent to removes leading indentation from ES6 template strings
                            Expression::TaggedTemplateExpression(tag_expr) => {
                                let Expression::Identifier(ident) = &tag_expr.tag else {
                                    continue;
                                };
                                if ident.name != "dedent" && ident.name != "outdent" {
                                    continue;
                                }
                                tag_expr.quasi.quasi().map(ToString::to_string)
                            }
                            Expression::TemplateLiteral(tag_expr) => {
                                tag_expr.quasi().map(ToString::to_string)
                            }
                            // handle code like ["{", "a: 1", "}"].join("\n")
                            Expression::CallExpression(call_expr) => {
                                if !call_expr.arguments.first().is_some_and(|arg|  matches!(arg, Argument::Expression(Expression::StringLiteral(string)) if string.value == "\n")) {
                                    continue;
                                }
                                let Expression::MemberExpression(member_expr) = &call_expr.callee
                                else {
                                    continue;
                                };
                                let MemberExpression::StaticMemberExpression(member) =
                                    &member_expr.0
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
                                            ArrayExpressionElement::Expression(
                                                Expression::StringLiteral(string),
                                            ) => string.value.as_str(),
                                            _ => "",
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                )
                            }
                            _ => continue,
                        }
                    }
                    PropertyKey::Identifier(ident) if ident.name == "options" => {
                        let span = prop.value.span();
                        let option_text = &self.source_text[span.start as usize..span.end as usize];
                        self.config =
                            Some(Cow::Owned(json::convert_config_to_json_literal(option_text)));
                    }
                    PropertyKey::Identifier(ident) if ident.name == "settings" => {
                        let span = prop.value.span();
                        let setting_text =
                            &self.source_text[span.start as usize..span.end as usize];
                        self.settings =
                            Some(Cow::Owned(json::convert_config_to_json_literal(setting_text)));
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
        let Expression::Identifier(ident) = &expr.tag else {
            return;
        };
        if ident.name != "dedent" && ident.name != "outdent" {
            return;
        }
        self.code = expr.quasi.quasi().map(std::string::ToString::to_string);
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
        }
    }
}

struct State<'a> {
    source_text: &'a str,
    valid_tests: Vec<&'a Expression<'a>>,
    invalid_tests: Vec<&'a Expression<'a>>,
    expression_to_group_comment_map: HashMap<Span, String>,
    group_comment_stack: Vec<String>,
}

impl<'a> State<'a> {
    fn new(source_text: &'a str) -> Self {
        Self {
            source_text,
            valid_tests: vec![],
            invalid_tests: vec![],
            expression_to_group_comment_map: HashMap::new(),
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
            Statement::ModuleDeclaration(mod_decl) => {
                if let ModuleDeclaration::ExportDefaultDeclaration(export_decl) = &mod_decl.0 {
                    if let ExportDefaultDeclarationKind::Expression(Expression::ObjectExpression(
                        obj_expr,
                    )) = &export_decl.declaration
                    {
                        self.visit_object_expression(obj_expr);
                    }
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
                if let Some(Argument::Expression(Expression::StringLiteral(lit))) =
                    expr.arguments.first()
                {
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
        let PropertyKey::Identifier(ident) = &prop.key else { return };
        match ident.name.as_str() {
            "valid" => {
                if let Expression::ArrayExpression(array_expr) = &prop.value {
                    let array_expr = self.alloc(array_expr);
                    for arg in &array_expr.elements {
                        if let ArrayExpressionElement::Expression(expr) = arg {
                            self.add_valid_test(expr);
                        }
                    }
                }

                // for eslint-plugin-jsx-a11y
                if let Some(args) = find_parser_arguments(&prop.value).map(|args| self.alloc(args))
                {
                    for arg in args {
                        if let Argument::Expression(expr) = arg {
                            self.add_valid_test(expr);
                        }
                    }
                }

                if let Expression::CallExpression(call_expr) = &prop.value {
                    if let Expression::MemberExpression(_) = &call_expr.callee {
                        // for eslint-plugin-react
                        if let Some(Argument::Expression(Expression::ArrayExpression(array_expr))) =
                            call_expr.arguments.first()
                        {
                            let array_expr = self.alloc(array_expr);
                            for arg in &array_expr.elements {
                                if let ArrayExpressionElement::Expression(expr) = arg {
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
                        if let ArrayExpressionElement::Expression(expr) = arg {
                            self.add_invalid_test(expr);
                        }
                    }
                }

                // for eslint-plugin-jsx-a11y
                if let Some(args) = find_parser_arguments(&prop.value).map(|args| self.alloc(args))
                {
                    for arg in args {
                        if let Argument::Expression(expr) = arg {
                            self.add_invalid_test(expr);
                        }
                    }
                }

                // for eslint-plugin-react
                if let Expression::CallExpression(call_expr) = &prop.value {
                    if let Expression::MemberExpression(_) = &call_expr.callee {
                        if let Some(Argument::Expression(Expression::ArrayExpression(array_expr))) =
                            call_expr.arguments.first()
                        {
                            let array_expr = self.alloc(array_expr);
                            for arg in &array_expr.elements {
                                if let ArrayExpressionElement::Expression(expr) = arg {
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
    expr: &'b Expression<'a>,
) -> Option<&'b oxc_allocator::Vec<'a, Argument<'a>>> {
    let Expression::CallExpression(call_expr) = expr else { return None };
    let Expression::MemberExpression(member_expr) = &call_expr.callee else {
        return None;
    };
    let MemberExpression::StaticMemberExpression(StaticMemberExpression {
        object, property, ..
    }) = &member_expr.0
    else {
        return None;
    };
    match (object, call_expr.arguments.first()) {
        (Expression::Identifier(iden), Some(Argument::Expression(arg)))
            if iden.name == "parsers" && property.name == "all" =>
        {
            if let Expression::CallExpression(call_expr) = arg {
                if let Expression::MemberExpression(_) = &call_expr.callee {
                    return Some(&call_expr.arguments);
                }
            }
            None
        }
        _ => find_parser_arguments(object),
    }
}

#[derive(Clone, Copy)]
pub enum RuleKind {
    ESLint,
    Jest,
    Typescript,
    Unicorn,
    React,
    ReactPerf,
    JSXA11y,
    Oxc,
    DeepScan,
    NextJS,
    JSDoc,
    Node,
    TreeShaking,
}

impl RuleKind {
    fn from(kind: &str) -> Self {
        match kind {
            "jest" => Self::Jest,
            "typescript" => Self::Typescript,
            "unicorn" => Self::Unicorn,
            "react" => Self::React,
            "react-perf" => Self::ReactPerf,
            "jsx-a11y" => Self::JSXA11y,
            "oxc" => Self::Oxc,
            "deepscan" => Self::DeepScan,
            "nextjs" => Self::NextJS,
            "jsdoc" => Self::JSDoc,
            "n" => Self::Node,
            "tree-shaking" => Self::TreeShaking,
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
            Self::React => write!(f, "eslint-plugin-react"),
            Self::ReactPerf => write!(f, "eslint-plugin-react-perf"),
            Self::JSXA11y => write!(f, "eslint-plugin-jsx-a11y"),
            Self::DeepScan => write!(f, "deepscan"),
            Self::Oxc => write!(f, "oxc"),
            Self::NextJS => write!(f, "eslint-plugin-next"),
            Self::JSDoc => write!(f, "eslint-plugin-jsdoc"),
            Self::Node => write!(f, "eslint-plugin-n"),
            Self::TreeShaking => write!(f, "eslint-plugin-tree-shaking"),
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
        RuleKind::React => format!("{REACT_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::ReactPerf => format!("{REACT_PERF_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::JSXA11y => format!("{JSX_A11Y_TEST_PATH}/{kebab_rule_name}-test.js"),
        RuleKind::NextJS => format!("{NEXT_JS_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::JSDoc => format!("{JSDOC_TEST_PATH}/{camel_rule_name}.js"),
        RuleKind::Node => format!("{NODE_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::TreeShaking => format!("{TREE_SHAKING_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::Oxc | RuleKind::DeepScan => String::new(),
    };

    println!("Reading test file from {rule_test_path}");

    let body = oxc_tasks_common::agent().get(&rule_test_path).call().map(Response::into_string);
    let context = match body {
        Ok(Ok(body)) => {
            let allocator = Allocator::default();
            let source_type = SourceType::from_path(rule_test_path).expect("incorrect {path:?}");
            let ret = Parser::new(&allocator, &body, source_type).parse();

            let program = allocator.alloc(ret.program);

            let mut state = State::new(&body);
            state.visit_program(program);

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

            let gen_cases_string = |cases: Vec<TestCase>| {
                let mut codes = vec![];
                let mut last_comment = String::new();
                for case in cases {
                    let current_comment = case.group_comment();
                    let mut code = case.code(has_config, has_settings);
                    if code.is_empty() {
                        continue;
                    }
                    if let Some(current_comment) = current_comment {
                        if current_comment != last_comment {
                            last_comment = current_comment.to_string();
                            code = format!(
                                "// {}\n{}",
                                &last_comment,
                                case.code(has_config, has_settings)
                            );
                        }
                    }

                    codes.push(code);
                }

                codes.join(",\n")
            };

            let pass_cases = gen_cases_string(pass_cases);
            let fail_cases = gen_cases_string(fail_cases);

            Context::new(plugin_name, &rule_name, pass_cases, fail_cases)
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
