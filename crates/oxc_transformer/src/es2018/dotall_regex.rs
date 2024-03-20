use std::rc::Rc;
use lazy_static::lazy_static;

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::SPAN;
use regex_syntax::ast::parse::Parser;
use regex_syntax::ast::{Ast};

use crate::options::{TransformOptions, TransformTarget};

/// ES2018: Dotall Regular Expression
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-dotall-regex>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-dotall-regex>
pub struct DotallRegex<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> DotallRegex<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, options: &TransformOptions) -> Option<Self> {
        (options.target < TransformTarget::ES2018 || options.dotall_regex).then_some(Self { ast })
    }

    pub fn transform_expression(&self, expr: &mut Expression<'a>) {
        let Expression::RegExpLiteral(literal) = expr else {
            return;
        };
        if !literal.regex.flags.contains(RegExpFlags::S) {
            return;
        }
        literal.regex.flags.remove(RegExpFlags::S);
        let mut parser = Parser::new();
        if let Ok(mut pattern) = parser.parse(literal.regex.pattern.as_str()) {
            Self::transform_dot(&mut pattern);
            let pattern = self.ast.new_str(pattern.to_string().as_str());
            let regex = self.ast.reg_exp_literal(SPAN, pattern, literal.regex.flags);
            *expr = self.ast.literal_regexp_expression(regex);
        }
    }

    fn transform_dot(ast: &mut Ast) {
        lazy_static! {
            static ref DOTALL: Ast = {
                let mut parser = Parser::new();
                parser.parse("[\\s\\S]").unwrap()
            };
        }
        match ast {
            Ast::Dot(_) => {
                *ast = DOTALL.clone();
            }
            Ast::Concat(ref mut concat) => {
                concat.asts.iter_mut().for_each(Self::transform_dot);
            }
            Ast::Group(ref mut group) => {
                Self::transform_dot(&mut group.ast);
            }
            Ast::Alternation(ref mut alternation) => {
                alternation.asts.iter_mut().for_each(Self::transform_dot);
            }
            Ast::Repetition(ref mut repetition) => {
                Self::transform_dot(&mut repetition.ast);
            }
            _ => {}
        }
    }
}
