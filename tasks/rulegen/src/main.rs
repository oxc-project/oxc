use std::{
    borrow::Cow,
    fmt,
    fmt::{Display, Formatter},
};

use convert_case::{Case, Casing};
use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{
        Argument, ArrayExpression, ArrayExpressionElement, CallExpression, Expression,
        ExpressionStatement, MemberExpression, ObjectExpression, ObjectProperty,
        ObjectPropertyKind, Program, PropertyKey, Statement, StringLiteral,
        TaggedTemplateExpression, TemplateLiteral,
    },
    Visit,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
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

struct TestCase<'a> {
    source_text: &'a str,
    code: Option<String>,
    test_code: Option<Cow<'a, str>>,
}

impl<'a> TestCase<'a> {
    fn new(source_text: &'a str, arg: &'a ArrayExpressionElement<'a>) -> Option<Self> {
        let mut test_case = TestCase { source_text, code: None, test_code: None };
        if let ArrayExpressionElement::Expression(expr) = arg {
            test_case.visit_expression(expr);
            return Some(test_case);
        }
        None
    }

    fn code(&self) -> Option<Cow<'a, str>> {
        self.code.as_ref().map(|test_code| {
            let option_code =
                self.test_code.as_ref().map_or(Cow::Borrowed("None"), |option_code| {
                    Cow::Owned(format!("Some(serde_json::json!({option_code}))"))
                });
            if test_code.contains('\n') {
                Cow::Owned(format!(
                    r#"("{}", {option_code})"#,
                    test_code.replace('\n', "\n\t\t\t").replace('\\', "\\\\").replace('\"', "\\\"")
                ))
            } else {
                Cow::Owned(format!(r#"({test_code:?}, {option_code})"#))
            }
        })
    }

    fn to_code(test_case: &TestCase) -> String {
        test_case.code().map_or_else(String::new, |code| code.clone().into_owned())
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
                self.test_code = None;
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
                            Expression::TaggedTemplateExpression(tag_expr) => {
                                let Expression::Identifier(ident) = &tag_expr.tag else {
                                    continue;
                                };
                                if ident.name != "dedent" {
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
                        self.test_code =
                            Some(Cow::Owned(json::wrap_property_in_quotes(option_text)));
                    }
                    _ => continue,
                },
                ObjectPropertyKind::SpreadProperty(_) => continue,
            }
        }
    }

    fn visit_template_literal(&mut self, lit: &TemplateLiteral<'a>) {
        self.code = Some(lit.quasi().unwrap().to_string());
        self.test_code = None;
    }

    fn visit_string_literal(&mut self, lit: &StringLiteral) {
        self.code = Some(lit.value.to_string());
        self.test_code = None;
    }

    fn visit_tagged_template_expression(&mut self, expr: &TaggedTemplateExpression<'a>) {
        let Expression::Identifier(ident) = &expr.tag else {
            return;
        };
        if ident.name != "dedent" {
            return;
        }
        self.code = expr.quasi.quasi().map(std::string::ToString::to_string);
        self.test_code = None;
    }
}

#[derive(Serialize)]
pub struct Context<'a> {
    rule: &'a str,
    rule_name: &'a str,
    pass_cases: &'a str,
    fail_cases: &'a str,
}

impl<'a> Context<'a> {
    fn new(
        upper_rule_name: &'a str,
        rule_name: &'a str,
        pass_cases: &'a str,
        fail_cases: &'a str,
    ) -> Self {
        Context { rule: upper_rule_name, rule_name, pass_cases, fail_cases }
    }
}

struct State<'a> {
    source_text: &'a str,
    valid_tests: Vec<&'a ArrayExpression<'a>>,
    invalid_tests: Vec<&'a ArrayExpression<'a>>,
}

impl<'a> State<'a> {
    fn new(source_text: &'a str) -> Self {
        Self { source_text, valid_tests: vec![], invalid_tests: vec![] }
    }

    fn pass_cases(&self) -> Vec<TestCase> {
        self.valid_tests
            .iter()
            .flat_map(|array_expr| (&array_expr.elements).into_iter())
            .filter_map(|arg| TestCase::new(self.source_text, arg))
            .collect::<Vec<_>>()
    }

    fn fail_cases(&self) -> Vec<TestCase> {
        self.invalid_tests
            .iter()
            .flat_map(|array_expr| (&array_expr.elements).into_iter())
            .filter_map(|arg| TestCase::new(self.source_text, arg))
            .collect::<Vec<_>>()
    }
}

impl<'a> Visit<'a> for State<'a> {
    fn visit_program(&mut self, program: &Program<'a>) {
        for stmt in &program.body {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &Statement<'a>) {
        if let Statement::ExpressionStatement(expr_stmt) = stmt {
            self.visit_expression_statement(expr_stmt);
        }
    }

    fn visit_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) {
        self.visit_expression(&stmt.expression);
    }

    fn visit_expression(&mut self, expr: &Expression<'a>) {
        if let Expression::CallExpression(call_expr) = expr {
            for arg in &call_expr.arguments {
                self.visit_argument(arg);
            }
        }
    }

    fn visit_argument(&mut self, arg: &Argument<'a>) {
        if let Argument::Expression(Expression::ObjectExpression(obj_expr)) = arg {
            for obj_prop in &obj_expr.properties {
                let ObjectPropertyKind::ObjectProperty(prop) = obj_prop else { return };
                self.visit_object_property(prop);
            }
        }
    }

    fn visit_object_property(&mut self, prop: &ObjectProperty<'a>) {
        let PropertyKey::Identifier(ident) = &prop.key else { return };
        match ident.name.as_str() {
            "valid" => {
                if let Expression::ArrayExpression(array_expr) = &prop.value {
                    self.valid_tests.push(self.alloc(array_expr));
                }
            }
            "invalid" => {
                if let Expression::ArrayExpression(array_expr) = &prop.value {
                    self.invalid_tests.push(self.alloc(array_expr));
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
pub enum RuleKind {
    ESLint,
    Jest,
    Typescript,
    Unicorn,
}

impl RuleKind {
    fn from(kind: &str) -> Self {
        match kind {
            "jest" => Self::Jest,
            "typescript" => Self::Typescript,
            "unicorn" => Self::Unicorn,
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
        }
    }
}

fn main() {
    let mut args = std::env::args();
    args.next();

    let rule_name = args.next().expect("expected rule name").to_case(Case::Snake);
    let rule_kind = args.next().map_or(RuleKind::ESLint, |kind| RuleKind::from(&kind));
    let upper_rule_name = rule_name.to_case(Case::UpperCamel);
    let kebab_rule_name = rule_name.to_case(Case::Kebab);

    let rule_test_path = match rule_kind {
        RuleKind::ESLint => format!("{ESLINT_TEST_PATH}/{kebab_rule_name}.js"),
        RuleKind::Jest => format!("{JEST_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::Typescript => format!("{TYPESCRIPT_ESLINT_TEST_PATH}/{kebab_rule_name}.test.ts"),
        RuleKind::Unicorn => format!("{UNICORN_TEST_PATH}/{kebab_rule_name}.mjs"),
    };
    println!("Reading test file from {rule_test_path}");

    let body = oxc_tasks_common::agent().get(&rule_test_path).call().map(Response::into_string);
    let pass_cases;
    let fail_cases;
    let context = match body {
        Ok(Ok(body)) => {
            let allocator = Allocator::default();
            let source_type = SourceType::from_path(rule_test_path).expect("incorrect {path:?}");
            let ret = Parser::new(&allocator, &body, source_type).parse();

            let program = allocator.alloc(ret.program);

            let mut state = State::new(&body);
            state.visit_program(program);

            pass_cases = state
                .pass_cases()
                .iter()
                .map(TestCase::to_code)
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
                .join(",\n");
            fail_cases = state
                .fail_cases()
                .iter()
                .map(TestCase::to_code)
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
                .join(",\n");

            Context::new(&upper_rule_name, &rule_name, &pass_cases, &fail_cases)
        }
        Err(_err) => {
            println!("Rule {rule_name} cannot be found in {rule_kind}, use empty template.");
            Context::new(&upper_rule_name, &rule_name, "", "")
        }
        Ok(Err(err)) => {
            println!("Failed to convert rule source code to string: {err}, use empty template");
            Context::new(&upper_rule_name, &rule_name, "", "")
        }
    };

    let template = template::Template::with_context(&context);
    if let Err(err) = template.render(rule_kind) {
        let rule_name = context.rule;
        eprintln!("failed to render {rule_name} rule template: {err}");
    }
}
