use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, CallExpression, Expression, ObjectPropertyKind, Program,
    },
    VisitMut,
};
use oxc_parser::Parser;
use oxc_prettier::{PrettierOptions, TrailingComma};
use oxc_span::{Atom, SourceType};

#[derive(Default)]
pub struct SpecParser {
    pub calls: Vec<(PrettierOptions, Vec<(Atom, String)>)>,
    source_text: String,
}

impl SpecParser {
    pub fn parse(&mut self, spec: &Path) {
        let spec_content = fs::read_to_string(spec).unwrap_or_default();
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(spec).unwrap_or_default();
        let mut ret = Parser::new(&allocator, &spec_content, source_type).parse();
        self.source_text = spec_content.clone();
        self.visit_program(&mut ret.program);
    }
}

impl VisitMut<'_> for SpecParser {
    fn visit_program(&mut self, program: &mut Program<'_>) {
        self.visit_statements(&mut program.body);
    }

    fn visit_call_expression(&mut self, expr: &mut CallExpression<'_>) {
        let Some(ident) = expr.callee.get_identifier_reference() else { return };
        if ident.name != "run_spec" {
            return;
        }
        let mut parsers = vec![];
        let mut snapshot_options = vec![];
        let mut options = PrettierOptions::default();

        if let Some(argument) = expr.arguments.get(1) {
            if let Argument::Expression(Expression::ArrayExpression(arr_expr)) = argument {
                parsers = arr_expr
                    .elements
                    .iter()
                    .filter_map(|el| {
                        if let ArrayExpressionElement::Expression(Expression::StringLiteral(
                            literal,
                        )) = el
                        {
                            return Some(literal.value.to_string());
                        }
                        None
                    })
                    .collect::<Vec<String>>();
            }
        } else {
            return;
        }

        if let Some(Argument::Expression(Expression::ObjectExpression(obj_expr))) =
            expr.arguments.get(2)
        {
            obj_expr.properties.iter().for_each(|item| {
                if let ObjectPropertyKind::ObjectProperty(obj_prop) = item {
                    if let Some(name) = obj_prop.key.static_name() {
                        match &obj_prop.value {
                            Expression::BooleanLiteral(literal) => {
                                if name == "semi" {
                                    options.semi = literal.value;
                                }
                                snapshot_options.push((name, literal.value.to_string()));
                            }
                            Expression::NumberLiteral(literal) => {
                                if name == "printWidth" {
                                    options.print_width =
                                        str::parse(&literal.value.to_string()).unwrap_or(80);
                                }
                                snapshot_options.push((name, literal.value.to_string()));
                            }
                            Expression::StringLiteral(literal) => {
                                if name == "trailingComma" {
                                    options.trailing_comma = match literal.value.as_str() {
                                        "none" => TrailingComma::None,
                                        "es5" => TrailingComma::ES5,
                                        _ => TrailingComma::All,
                                    }
                                }
                                snapshot_options.push((name, format!("\"{}\"", literal.value)));
                            }
                            _ => {}
                        };
                    };
                }
            });
        }

        snapshot_options.push((
            "parsers".into(),
            format!(
                "[{}]",
                parsers.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ")
            ),
        ));
        if !snapshot_options.iter().any(|item| item.0 == "printWidth") {
            snapshot_options.push(("printWidth".into(), "80".into()));
        }

        snapshot_options.sort_by(|a, b| a.0.cmp(&b.0));

        self.calls.push((options, snapshot_options));
    }
}
