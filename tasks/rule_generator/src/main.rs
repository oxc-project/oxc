use std::{borrow::Cow, fs::File, io::Write, path::Path, process::Command, rc::Rc};

use convert_case::{Case, Casing};
use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{Argument, Expression, ObjectProperty, PropertyKey, PropertyValue},
    AstKind, SourceType,
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use serde::Serialize;

const RULE_TEMPLATE: &str = include_str!("../template.txt");
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
            TestCase::Valid(code) | TestCase::Invalid(code) => format!(r#"("{code}", None)"#),
        }
    }
}

#[derive(Serialize)]
struct Context<'a> {
    rule: &'a str,
    pass_cases: &'a str,
    fail_cases: &'a str,
}

fn parse_test_code<'a>(expr: &'a Expression) -> Option<Cow<'a, str>> {
    match expr {
        Expression::StringLiteral(lit) => Some(Cow::Borrowed(lit.value.as_str())),
        Expression::TemplateLiteral(lit) => Some(Cow::Borrowed(lit.quasi().unwrap().as_str())),
        Expression::ObjectExpression(obj_expr) => {
            for obj_prop in &obj_expr.properties {
                match obj_prop {
                    ObjectProperty::Property(prop) => match &prop.key {
                        PropertyKey::Identifier(ident) if ident.name == "code" => match &prop.value
                        {
                            PropertyValue::Expression(expr) => return parse_test_code(expr),
                            PropertyValue::Pattern(_) => continue,
                        },
                        _ => continue,
                    },
                    ObjectProperty::SpreadProperty(_) => continue,
                }
            }
            None
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
                    Some(Cow::Owned(code))
                }
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    let rule_name = args.next().expect("expected rule name");
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
    let trivias = Rc::new(ret.trivias);
    let semantic = SemanticBuilder::new(source_type).build(program, trivias);

    let tests_object = semantic.nodes().iter().find_map(|node| match node.get().kind() {
        AstKind::ExpressionStatement(stmt) => match &stmt.expression {
            Expression::CallExpression(call_expr) => {
                for arg in &call_expr.arguments {
                    match arg {
                        Argument::Expression(Expression::ObjectExpression(obj_expr)) => {
                            let mut tests = (None, None);
                            for obj_prop in &obj_expr.properties {
                                let ObjectProperty::Property(prop) = obj_prop else { return None };
                                let PropertyKey::Identifier(ident) = &prop.key else { return None };
                                match ident.name.as_str() {
                                    "valid" => match &prop.value {
                                        PropertyValue::Expression(Expression::ArrayExpression(
                                            array_expr,
                                        )) => tests.0 = Some(array_expr),
                                        _ => continue,
                                    },
                                    "invalid" => match &prop.value {
                                        PropertyValue::Expression(Expression::ArrayExpression(
                                            array_expr,
                                        )) => tests.1 = Some(array_expr),
                                        _ => continue,
                                    },
                                    _ => continue,
                                }
                            }
                            if tests.0.is_some() && tests.1.is_some() {
                                return Some(tests);
                            }
                        }
                        _ => continue,
                    }
                }

                None
            }
            _ => None,
        },
        _ => None,
    });

    let mut pass_cases = vec![];
    let mut fail_cases = vec![];
    if let Some((Some(valid), Some(invalid))) = tests_object {
        for arg in (&valid.elements).into_iter().flatten() {
            if let Argument::Expression(expr) = arg {
                let Some(code) = parse_test_code(expr) else { continue };
                pass_cases.push(TestCase::Valid(code));
            }
        }

        for arg in (&invalid.elements).into_iter().flatten() {
            if let Argument::Expression(expr) = arg {
                let Some(code) = parse_test_code(expr) else { continue };
                fail_cases.push(TestCase::Invalid(code));
            }
        }
    }

    let mut eng = handlebars::Handlebars::new();
    eng.register_escape_fn(handlebars::no_escape);

    let rule = rule_name.to_case(Case::UpperCamel);
    let context = Context {
        rule: &rule,
        pass_cases: &pass_cases.iter().map(TestCase::to_code).collect::<Vec<_>>().join(",\n"),
        fail_cases: &fail_cases.iter().map(TestCase::to_code).collect::<Vec<_>>().join(",\n"),
    };
    let rendered = eng.render_template(RULE_TEMPLATE, &handlebars::to_json(context)).unwrap();

    let out_path = Path::new("crates/oxc_linter/src/rules")
        .join(format!("{}.rs", rule_name.to_case(Case::Snake)));
    let mut out_file = File::create(out_path.clone()).expect("failed to create output file");
    out_file.write_all(rendered.as_bytes()).expect("failed to write output");

    Command::new("cargo")
        .arg("fmt")
        .arg("--")
        .arg(out_path)
        .spawn()
        .expect("failed to format output");
}
