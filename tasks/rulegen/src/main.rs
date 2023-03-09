use std::borrow::Cow;

use convert_case::{Case, Casing};
use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrayExpression, ExpressionStatement, Program, Property, Statement};
use oxc_ast::visit::Visit;
use oxc_ast::{
    ast::{Argument, Expression, ObjectProperty, PropertyKey, PropertyValue},
    GetSpan, SourceType,
};
use oxc_parser::Parser;
use serde::Serialize;

mod json;
mod template;

const ESLINT_TEST_PATH: &str =
    "https://raw.githubusercontent.com/eslint/eslint/main/tests/lib/rules";

#[derive(Debug)]
enum TestCase<'a> {
    Invalid(Cow<'a, str>),
    Valid(Cow<'a, str>),
}

impl<'a> TestCase<'a> {
    fn to_code(test_case: &TestCase) -> String {
        match test_case {
            TestCase::Valid(code) | TestCase::Invalid(code) => code.clone().into_owned(),
        }
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
    valid_tests: Vec<&'a ArrayExpression<'a>>,
    invalid_tests: Vec<&'a ArrayExpression<'a>>,
}

impl<'a> State<'a> {
    fn new() -> Self {
        Self { valid_tests: vec![], invalid_tests: vec![] }
    }

    fn pass_cases<'b>(&'b self, body: &'b str) -> Vec<TestCase> {
        self.valid_tests
            .iter()
            .flat_map(|array_expr| (&array_expr.elements).into_iter().flatten())
            .filter_map(|arg| {
                if let Argument::Expression(expr) = arg {
                    let Some(code) = parse_test_code(body, expr) else { return None };
                    return Some(TestCase::Valid(code));
                }
                None
            })
            .collect::<Vec<_>>()
    }

    fn fail_cases<'b>(&'b self, body: &'b str) -> Vec<TestCase> {
        self.invalid_tests
            .iter()
            .flat_map(|array_expr| (&array_expr.elements).into_iter().flatten())
            .filter_map(|arg| {
                if let Argument::Expression(expr) = arg {
                    let Some(code) = parse_test_code(body, expr) else { return None };
                    return Some(TestCase::Invalid(code));
                }
                None
            })
            .collect::<Vec<_>>()
    }
}

impl<'a> Visit<'a> for State<'a> {
    fn visit_program(&mut self, program: &'a Program<'a>) {
        for stmt in &program.body {
            self.visit_statement(stmt);
        }
    }

    fn visit_statement(&mut self, stmt: &'a Statement<'a>) {
        if let Statement::ExpressionStatement(expr_stmt) = stmt {
            self.visit_expression_statement(expr_stmt);
        }
    }

    fn visit_expression_statement(&mut self, stmt: &'a ExpressionStatement<'a>) {
        self.visit_expression(&stmt.expression);
    }

    fn visit_expression(&mut self, expr: &'a Expression<'a>) {
        if let Expression::CallExpression(call_expr) = expr {
            for arg in &call_expr.arguments {
                self.visit_argument(arg);
            }
        }
    }

    fn visit_argument(&mut self, arg: &'a Argument<'a>) {
        if let Argument::Expression(Expression::ObjectExpression(obj_expr)) = arg {
            for obj_prop in &obj_expr.properties {
                let ObjectProperty::Property(prop) = obj_prop else { return };
                self.visit_property(prop);
            }
        }
    }

    fn visit_property(&mut self, prop: &'a Property<'a>) {
        let PropertyKey::Identifier(ident) = &prop.key else { return };
        match ident.name.as_str() {
            "valid" => {
                if let PropertyValue::Expression(Expression::ArrayExpression(array_expr)) =
                    &prop.value
                {
                    self.valid_tests.push(array_expr);
                }
            }
            "invalid" => {
                if let PropertyValue::Expression(Expression::ArrayExpression(array_expr)) =
                    &prop.value
                {
                    self.invalid_tests.push(array_expr);
                }
            }
            _ => {}
        }
    }
}

fn parse_test_code<'a>(source_text: &'a str, expr: &'a Expression) -> Option<Cow<'a, str>> {
    let (test_code, option_code) = match expr {
        Expression::StringLiteral(lit) => (Some(Cow::Borrowed(lit.value.as_str())), None),
        Expression::TemplateLiteral(lit) => {
            (Some(Cow::Borrowed(lit.quasi().unwrap().as_str())), None)
        }
        Expression::ObjectExpression(obj_expr) => {
            let mut test_code = None;
            let mut option_code: Option<Cow<'_, str>> = None;
            for obj_prop in &obj_expr.properties {
                match obj_prop {
                    ObjectProperty::Property(prop) => match &prop.key {
                        PropertyKey::Identifier(ident) if ident.name == "code" => match &prop.value
                        {
                            PropertyValue::Expression(expr) => {
                                let Expression::StringLiteral(s) = expr else {
                                return None;
                              };
                                test_code = Some(Cow::Borrowed(s.value.as_str()));
                            }
                            PropertyValue::Pattern(_) => continue,
                        },
                        PropertyKey::Identifier(ident) if ident.name == "options" => {
                            let span = prop.value.span();
                            let option_text = &source_text[span.start as usize..span.end as usize];
                            option_code =
                                Some(Cow::Owned(json::wrap_property_in_quotes(option_text)));
                        }
                        _ => continue,
                    },
                    ObjectProperty::SpreadProperty(_) => continue,
                }
            }
            (test_code, option_code)
        }
        Expression::CallExpression(call_expr) => match &call_expr.callee {
            Expression::MemberExpression(member_expr) => match &member_expr.object() {
                // ['class A {', '}'].join('\n')
                Expression::ArrayExpression(array_expr) => {
                    let mut code = String::new();
                    for arg in &array_expr.elements {
                        let Some(Argument::Expression(Expression::StringLiteral(lit))) = arg else { continue };
                        code.push_str(lit.value.as_str());
                        code.push('\n');
                    }
                    (Some(Cow::Owned(code)), None)
                }
                _ => (None, None),
            },
            _ => (None, None),
        },
        _ => (None, None),
    };

    test_code.map(|test_code| {
        let option_code = option_code.map_or(Cow::Borrowed("None"), |option_code| {
            Cow::Owned(format!("Some(serde_json::json!({option_code}))"))
        });
        Cow::Owned(format!(r#"({test_code:?}, {option_code})"#))
    })
}

fn main() {
    let mut args = std::env::args();
    let _ = args.next();

    let rule_name = args.next().expect("expected rule name");
    let upper_rule_name = rule_name.to_case(Case::UpperCamel);

    let rule_test_path = format!("{ESLINT_TEST_PATH}/{rule_name}.js");
    let body = ureq::get(&rule_test_path)
        .call()
        .expect("failed to fetch source")
        .into_string()
        .expect("failed to read response as string");
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(rule_test_path).expect("incorrect {path:?}");
    let ret = Parser::new(&allocator, &body, source_type).parse();

    let program = allocator.alloc(ret.program);

    let mut state = State::new();
    state.visit_program(program);

    let pass_cases =
        state.pass_cases(&body).iter().map(TestCase::to_code).collect::<Vec<_>>().join(",\n");
    let fail_cases =
        state.fail_cases(&body).iter().map(TestCase::to_code).collect::<Vec<_>>().join(",\n");

    let context = Context::new(&upper_rule_name, &rule_name, &pass_cases, &fail_cases);
    let template = template::Template::with_context(&context);
    if template.render().is_err() {
        eprintln!("failed to render {} rule template", context.rule);
    }
}
