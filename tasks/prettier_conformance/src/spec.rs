#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::{fs, path::Path, str::FromStr};

use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{Argument, ArrayExpressionElement, CallExpression, Expression, ObjectPropertyKind},
    VisitMut,
};
use oxc_parser::Parser;
use oxc_prettier::{EndOfLine, PrettierOptions, TrailingComma};
use oxc_span::{Atom, GetSpan, SourceType};

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
        self.calls = vec![];
        self.visit_program(&mut ret.program);
    }
}

impl VisitMut<'_> for SpecParser {
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
                                } else if name == "bracketSpacing" {
                                    options.bracket_spacing = literal.value;
                                } else if name == "singleQuote" {
                                    options.single_quote = literal.value;
                                }
                            }
                            Expression::NumberLiteral(literal) => match name.as_str() {
                                "printWidth" => options.print_width = literal.value as usize,
                                "tabWidth" => options.tab_width = literal.value as usize,
                                _ => {}
                            },
                            Expression::StringLiteral(literal) => match name.as_str() {
                                "trailingComma" => {
                                    options.trailing_comma =
                                        TrailingComma::from_str(literal.value.as_str()).unwrap();
                                }
                                "endOfLine" => {
                                    options.end_of_line =
                                        EndOfLine::from_str(literal.value.as_str()).unwrap();
                                }
                                _ => {}
                            },
                            _ => {}
                        };
                        if name != "errors" {
                            snapshot_options.push((
                                name,
                                obj_prop.value.span().source_text(&self.source_text).to_string(),
                            ));
                        }
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
