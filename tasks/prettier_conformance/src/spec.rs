use std::{fs, path::Path, str::FromStr};

use oxc_allocator::Allocator;
use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, CallExpression, Expression, ObjectPropertyKind,
        VariableDeclarator,
    },
    VisitMut,
};
use oxc_parser::Parser;
use oxc_prettier::{ArrowParens, EndOfLine, PrettierOptions, QuoteProps, TrailingComma};
use oxc_span::{GetSpan, SourceType};

/// Vec<(key, value)>
type SnapshotOptions = Vec<(String, String)>;

pub fn parse_spec(spec: &Path) -> Vec<(PrettierOptions, SnapshotOptions)> {
    let mut parser = SpecParser::default();
    parser.parse(spec);
    parser.calls
}

#[derive(Default)]
struct SpecParser {
    source_text: String,
    parsers: Vec<String>,
    calls: Vec<(PrettierOptions, SnapshotOptions)>,
}

impl SpecParser {
    fn parse(&mut self, spec: &Path) {
        let spec_content = fs::read_to_string(spec).unwrap_or_default();

        self.source_text.clone_from(&spec_content);

        let allocator = Allocator::default();
        let source_type = SourceType::from_path(spec).unwrap_or_default();

        let mut ret = Parser::new(&allocator, &spec_content, source_type).parse();
        self.visit_program(&mut ret.program);
    }
}

impl VisitMut<'_> for SpecParser {
    // Some test cases use a variable to store the parsers.
    //
    // ```js
    // const parser = ["babel"];
    //
    // runFormatTest(import.meta, parser, {});
    // runFormatTest(import.meta, parser, { semi: false });
    // ````
    fn visit_variable_declarator(&mut self, decl: &mut VariableDeclarator<'_>) {
        let Some(name) = decl.id.get_identifier_name() else { return };
        if !matches!(name.as_str(), "parser" | "parsers") {
            return;
        }

        debug_assert!(self.parsers.is_empty(), "`parsers` is already defined");
        if let Some(Expression::ArrayExpression(arr_expr)) = &decl.init {
            for el in &arr_expr.elements {
                if let ArrayExpressionElement::StringLiteral(literal) = el {
                    self.parsers.push(literal.value.to_string());
                }
            }
        }
    }

    // The `runFormatTest()` function is used on prettier's test cases.
    // We need to collect all calls and get the options and parsers.
    fn visit_call_expression(&mut self, expr: &mut CallExpression<'_>) {
        let Some(ident) = expr.callee.get_identifier_reference() else { return };
        if ident.name != "runFormatTest" {
            return;
        }

        let mut snapshot_options: SnapshotOptions = vec![];
        let mut parsers = vec![];
        let mut options = PrettierOptions::default();

        // Get parsers
        if let Some(argument) = expr.arguments.get(1) {
            let Some(argument_expr) = argument.as_expression() else {
                return;
            };

            // If inlined array
            if let Expression::ArrayExpression(arr_expr) = argument_expr {
                for el in &arr_expr.elements {
                    if let ArrayExpressionElement::StringLiteral(literal) = el {
                        parsers.push(literal.value.to_string());
                    }
                }
            }
            // If variable
            if let Expression::Identifier(_) = argument_expr {
                debug_assert!(
                    !self.parsers.is_empty(),
                    "`parsers` is not collected, check variable name"
                );
                parsers.clone_from(&self.parsers);
            }
        } else {
            return;
        }

        // Get options
        if let Some(Argument::ObjectExpression(obj_expr)) = expr.arguments.get(2) {
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
                            #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                            Expression::NumericLiteral(literal) => match name.as_ref() {
                                "printWidth" => options.print_width = literal.value as usize,
                                "tabWidth" => options.tab_width = literal.value as usize,
                                _ => {}
                            },
                            Expression::StringLiteral(literal) => match name.as_ref() {
                                "trailingComma" => {
                                    options.trailing_comma =
                                        TrailingComma::from_str(literal.value.as_str()).unwrap();
                                }
                                "endOfLine" => {
                                    options.end_of_line =
                                        EndOfLine::from_str(literal.value.as_str()).unwrap();
                                }
                                "quoteProps" => {
                                    options.quote_props =
                                        QuoteProps::from_str(literal.value.as_str()).unwrap();
                                }
                                "arrowParens" => {
                                    options.arrow_parens =
                                        ArrowParens::from_str(literal.value.as_str()).unwrap();
                                }
                                _ => {}
                            },
                            _ => {}
                        };
                        if name != "errors" {
                            snapshot_options.push((
                                name.to_string(),
                                obj_prop.value.span().source_text(&self.source_text).to_string(),
                            ));
                        }
                    };
                }
            });
        }

        debug_assert!(!parsers.is_empty(), "`parsers` should not be empty");
        snapshot_options.push((
            "parsers".to_string(),
            format!(
                "[{}]",
                parsers.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ")
            ),
        ));

        if !snapshot_options.iter().any(|item| item.0 == "printWidth") {
            snapshot_options.push(("printWidth".to_string(), "80".into()));
        }

        snapshot_options.sort_by(|a, b| a.0.cmp(&b.0));

        self.calls.push((options, snapshot_options));
    }
}
