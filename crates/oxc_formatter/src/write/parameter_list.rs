use oxc_ast::ast::*;

use crate::{
    formatter::{Format, FormatResult, Formatter, prelude::*, separated::FormatSeparatedIter},
    options::{FormatTrailingCommas, TrailingSeparator},
};

pub struct ParameterList<'a, 'b> {
    list: &'b FormalParameters<'a>,
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

impl<'a, 'b> ParameterList<'a, 'b> {
    pub fn with_layout(list: &'b FormalParameters<'a>, layout: ParameterLayout) -> Self {
        Self { list, layout: Some(layout) }
    }
}

impl<'a> Format<'a> for ParameterList<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self.layout {
            None | Some(ParameterLayout::Default | ParameterLayout::NoParameters) => {
                let has_trailing_rest = self.list.rest.is_some();

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
                join_parameter_list(&mut joiner, self.list, trailing_separator, source_text);
                joiner.finish()
            }
            Some(ParameterLayout::Hug) => {
                let mut join = f.join_with(space());
                join.entries(
                    FormatSeparatedIter::new(self.list.items.iter(), ",")
                        .with_trailing_separator(TrailingSeparator::Omit),
                );
                join.finish()
            }
        }
    }
}

fn join_parameter_list<'a, Separator>(
    joiner: &mut JoinNodesBuilder<'_, '_, 'a, Separator>,
    list: &FormalParameters<'a>,
    trailing_separator: TrailingSeparator,
    source_text: &str,
) where
    Separator: Format<'a>,
{
    let entries = FormatSeparatedIter::new(list.items.iter(), ",")
        .with_trailing_separator(trailing_separator)
        .zip(list.items.iter());
    for (formatted, param) in entries {
        joiner.entry(param.span, source_text, &formatted);
    }
}
