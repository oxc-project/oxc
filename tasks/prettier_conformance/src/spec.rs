use std::{fs, path::Path, str::FromStr};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Expression, ObjectPropertyKind,
    VariableDeclarator,
};
use oxc_ast_visit::VisitMut;
use oxc_formatter::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing, FormatOptions,
    IndentStyle, IndentWidth, LineEnding, LineWidth, OperatorPosition, QuoteProperties, QuoteStyle,
    Semicolons, TrailingCommas,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

/// Vec<(key, value)>
type SnapshotOptions = Vec<(String, String)>;

pub fn parse_spec(spec: &Path) -> Vec<(FormatOptions, SnapshotOptions)> {
    let mut parser = SpecParser::default();
    parser.parse(spec);
    parser.calls
}

#[derive(Default)]
struct SpecParser {
    source_text: String,
    parsers: Vec<String>,
    calls: Vec<(FormatOptions, SnapshotOptions)>,
}

impl SpecParser {
    fn parse(&mut self, spec: &Path) {
        let spec_content = fs::read_to_string(spec).unwrap_or_default();

        self.source_text.clone_from(&spec_content);

        let allocator = Allocator::default();
        let mut source_type = SourceType::from_path(spec).unwrap_or_default();
        if source_type.is_javascript() {
            source_type = source_type.with_jsx(true);
        }

        let mut ret = Parser::new(&allocator, &spec_content, source_type).parse();
        assert!(ret.errors.is_empty());
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
    // ```
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
        let mut options = FormatOptions {
            // Use Prettier's default printWidth(80) instead of our default(100)
            line_width: LineWidth::try_from(80).unwrap(),
            ..Default::default()
        };

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
                if let ObjectPropertyKind::ObjectProperty(obj_prop) = item
                    && let Some(name) = obj_prop.key.static_name()
                {
                    match &obj_prop.value {
                        Expression::BooleanLiteral(literal) => {
                            if name == "semi" {
                                options.semicolons = if literal.value {
                                    Semicolons::Always
                                } else {
                                    Semicolons::AsNeeded
                                }
                            } else if name == "bracketSpacing" {
                                options.bracket_spacing = BracketSpacing::from(literal.value);
                            } else if matches!(
                                name.as_ref(),
                                "jsxBracketSameLine" | "bracketSameLine"
                            ) && literal.value
                            {
                                options.bracket_same_line = BracketSameLine::from(literal.value);
                            } else if name == "singleQuote" {
                                options.quote_style = if literal.value {
                                    QuoteStyle::Single
                                } else {
                                    QuoteStyle::Double
                                };
                            } else if name == "jsxSingleQuote" {
                                options.jsx_quote_style = if literal.value {
                                    QuoteStyle::Single
                                } else {
                                    QuoteStyle::Double
                                };
                            } else if name == "useTabs" {
                                options.indent_style = if literal.value {
                                    IndentStyle::Tab
                                } else {
                                    IndentStyle::Space
                                };
                            } else if name == "experimentalTernaries" {
                                options.experimental_ternaries = literal.value;
                            } else if name == "singleAttributePerLine" {
                                options.attribute_position = if literal.value {
                                    AttributePosition::Multiline
                                } else {
                                    AttributePosition::Auto
                                };
                            }
                        }
                        #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                        Expression::NumericLiteral(literal) => match name.as_ref() {
                            "printWidth" => {
                                options.line_width =
                                    LineWidth::try_from(literal.value as u16).unwrap();
                            }
                            "tabWidth" => {
                                options.indent_width =
                                    IndentWidth::try_from(literal.value as u8).unwrap();
                            }
                            _ => {}
                        },
                        Expression::StringLiteral(literal) => {
                            let s = literal.value.as_str();
                            match name.as_ref() {
                                "trailingComma" => {
                                    options.trailing_commas = TrailingCommas::from_str(s).unwrap();
                                }
                                "endOfLine" => {
                                    // TODO: change `unwrap_or_default` to `unwrap`
                                    options.line_ending =
                                        LineEnding::from_str(s).unwrap_or_default();
                                }
                                "quoteProps" => {
                                    // TODO: change `unwrap_or_default` to `unwrap`
                                    options.quote_properties =
                                        QuoteProperties::from_str(s).unwrap_or_default();
                                }
                                "objectWrap" => {
                                    // TODO: change `unwrap_or_default` to `unwrap`
                                    options.bracket_spacing =
                                        BracketSpacing::from_str(s).unwrap_or_default();
                                }
                                "arrowParens" => {
                                    // TODO: change `unwrap_or_default` to `unwrap`
                                    options.arrow_parentheses = ArrowParentheses::from_str(
                                        // Prettier uses "avoid", but we use "as-needed"
                                        if s == "avoid" { "as-needed" } else { s },
                                    )
                                    .unwrap_or_default();
                                }
                                "experimentalOperatorPosition" => {
                                    // TODO: change `unwrap_or_default` to `unwrap`
                                    options.experimental_operator_position =
                                        OperatorPosition::from_str(s).unwrap_or_default();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    if name != "errors" {
                        snapshot_options.push((
                            name.to_string(),
                            obj_prop.value.span().source_text(&self.source_text).to_string(),
                        ));
                    }
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
