use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    formatter::{Format, FormatResult, Formatter, prelude::*, separated::FormatSeparatedIter},
    generated::ast_nodes::{AstNode, AstNodeIterator},
    options::{FormatTrailingCommas, TrailingSeparator},
};

enum Parameter<'a, 'b, 'c> {
    FormalParameter(&'c AstNode<'a, 'b, FormalParameter<'a>>),
    Rest(&'c AstNode<'a, 'b, BindingRestElement<'a>>),
}

impl GetSpan for Parameter<'_, '_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::FormalParameter(param) => param.span(),
            Self::Rest(e) => e.span(),
        }
    }
}

impl<'a> Format<'a> for Parameter<'a, '_, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::FormalParameter(param) => param.fmt(f),
            Self::Rest(e) => e.fmt(f),
        }
    }
}

struct FormalParametersIter<'a, 'b, 'c> {
    params: AstNodeIterator<'a, 'b, FormalParameter<'a>>,
    rest: Option<&'c AstNode<'a, 'b, BindingRestElement<'a>>>,
}

impl<'a, 'b, 'c> From<&'c AstNode<'a, 'b, FormalParameters<'a>>>
    for FormalParametersIter<'a, 'b, 'c>
{
    fn from(value: &'c AstNode<'a, 'b, FormalParameters<'a>>) -> Self {
        Self { params: value.items().into_iter(), rest: value.rest() }
    }
}

impl<'a, 'b, 'c> Iterator for FormalParametersIter<'a, 'b, 'c> {
    type Item = Parameter<'a, 'b, 'c>;

    fn next(&mut self) -> Option<Self::Item> {
        self.params
            .next()
            .map(|param| Parameter::FormalParameter(param))
            .or_else(|| self.rest.take().map(|rest| Parameter::Rest(rest)))
    }
}

pub struct ParameterList<'a, 'b, 'c> {
    list: &'c AstNode<'a, 'b, FormalParameters<'a>>,
    layout: Option<ParameterLayout>,
}

#[derive(Debug, Copy, Clone)]
pub enum ParameterLayout {
    /// ```javascript
    /// function test() {}
    /// ```
    NoParameters,

    /// Enforce that the opening and closing parentheses aren't separated from the first token of the parameter.
    /// For example, to enforce that the `{`  and `}` of an object expression are formatted on the same line
    /// as the `(` and `)` tokens even IF the object expression itself breaks across multiple lines.
    ///
    /// ```javascript
    /// function test({
    ///     aVeryLongObjectBinding,
    ///     thatContinuesAndExceeds,
    ///     theLineWidth
    /// }) {}
    /// ```
    Hug,

    /// The default layout formats all parameters on the same line if they fit or breaks after the `(`
    /// and before the `)`.
    ///
    /// ```javascript
    /// function test(
    ///     firstParameter,
    ///     secondParameter,
    ///     thirdParameter
    /// ) {}
    /// ```
    Default,
}

impl<'a, 'b, 'c> ParameterList<'a, 'b, 'c> {
    pub fn with_layout(
        list: &'c AstNode<'a, 'b, FormalParameters<'a>>,
        layout: ParameterLayout,
    ) -> Self {
        Self { list, layout: Some(layout) }
    }
}

impl<'a> Format<'a> for ParameterList<'a, '_, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self.layout {
            None | Some(ParameterLayout::Default | ParameterLayout::NoParameters) => {
                let has_trailing_rest = self.list.rest().is_some();

                // If it's a rest parameter, the assumption is no more
                // parameters could be added afterward, so no separator is
                // added there either.
                let trailing_separator = if has_trailing_rest {
                    TrailingSeparator::Disallowed
                } else {
                    FormatTrailingCommas::All.trailing_separator(f.options())
                };

                // TODO
                let has_modifiers = false; //
                //self.list.iter().any(|node| {
                // matches!(
                // node,
                // Ok(AnyParameter::AnyJsConstructorParameter(
                // AnyJsConstructorParameter::TsPropertyParameter(_),
                // ))
                // )
                // });
                let source_text = f.source_text();
                let mut joiner = if has_modifiers {
                    f.join_nodes_with_hardline()
                } else {
                    f.join_nodes_with_soft_line()
                };
                let entries = FormatSeparatedIter::new(FormalParametersIter::from(self.list), ",")
                    .with_trailing_separator(trailing_separator)
                    .zip(FormalParametersIter::from(self.list));
                for (formatted, param) in entries {
                    joiner.entry(param.span(), source_text, &formatted);
                }
                joiner.finish()
            }
            Some(ParameterLayout::Hug) => {
                let mut join = f.join_with(space());
                join.entries(
                    FormatSeparatedIter::new(self.list.items().into_iter(), ",")
                        .with_trailing_separator(TrailingSeparator::Omit),
                );
                join.finish()
            }
        }
    }
}
