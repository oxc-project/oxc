use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    formatter::{Format, FormatResult, Formatter, prelude::*, separated::FormatSeparatedIter},
    generated::ast_nodes::{AstNode, AstNodeIterator},
    options::{FormatTrailingCommas, TrailingSeparator},
};

enum Parameter<'a, 'b> {
    FormalParameter(&'b AstNode<'a, FormalParameter<'a>>),
    Rest(&'b AstNode<'a, BindingRestElement<'a>>),
}

impl GetSpan for Parameter<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::FormalParameter(param) => param.span(),
            Self::Rest(e) => e.span(),
        }
    }
}

impl<'a> Format<'a> for Parameter<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::FormalParameter(param) => param.fmt(f),
            Self::Rest(e) => e.fmt(f),
        }
    }
}

struct FormalParametersIter<'a, 'b> {
    params: AstNodeIterator<'a, FormalParameter<'a>>,
    rest: Option<&'b AstNode<'a, BindingRestElement<'a>>>,
}

impl<'a, 'b> From<&'b AstNode<'a, FormalParameters<'a>>> for FormalParametersIter<'a, 'b> {
    fn from(value: &'b AstNode<'a, FormalParameters<'a>>) -> Self {
        Self { params: value.items().iter(), rest: value.rest() }
    }
}

impl<'a, 'b> Iterator for FormalParametersIter<'a, 'b> {
    type Item = Parameter<'a, 'b>;

    fn next(&mut self) -> Option<Self::Item> {
        self.params
            .next()
            .map(Parameter::FormalParameter)
            .or_else(|| self.rest.take().map(Parameter::Rest))
    }
}

pub struct ParameterList<'a, 'b> {
    list: &'b AstNode<'a, FormalParameters<'a>>,
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
    pub fn with_layout(
        list: &'b AstNode<'a, FormalParameters<'a>>,
        layout: ParameterLayout,
    ) -> Self {
        Self { list, layout: Some(layout) }
    }
}

impl<'a> Format<'a> for ParameterList<'a, '_> {
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

                let has_modifiers = self.list.items.iter().any(FormalParameter::has_modifier);
                let source_text = f.source_text();

                let mut joiner = if has_modifiers {
                    f.join_nodes_with_hardline()
                } else {
                    f.join_nodes_with_soft_line()
                };
                for formatted in
                    FormatSeparatedIter::new(FormalParametersIter::from(self.list), ",")
                        .with_trailing_separator(trailing_separator)
                {
                    joiner.entry(formatted.element.span(), &formatted);
                }
                joiner.finish()
            }
            Some(ParameterLayout::Hug) => {
                let mut join = f.join_with(space());
                join.entries(
                    FormatSeparatedIter::new(self.list.items().iter(), ",")
                        .with_trailing_separator(TrailingSeparator::Omit),
                );
                join.finish()
            }
        }
    }
}

/// Returns `true` if parentheses can be safely avoided and the `arrow_parentheses` formatter option allows it
pub fn can_avoid_parentheses(
    arrow: &ArrowFunctionExpression<'_>,
    f: &mut Formatter<'_, '_>,
) -> bool {
    f.options().arrow_parentheses.is_as_needed()
        && arrow.params.items.len() == 1
        && arrow.params.rest.is_none()
        && arrow.type_parameters.is_none()
        && arrow.return_type.is_none()
        && {
            let param = arrow.params.items.first().unwrap();
            param.pattern.type_annotation.is_none()
                && !param.pattern.optional
                && param.pattern.kind.is_binding_identifier()
        }
        && !f.comments().has_comments_in_span(arrow.params.span)
}

pub fn should_hug_function_parameters<'a>(
    parameters: &AstNode<'a, FormalParameters<'a>>,
    parentheses_not_needed: bool,
    f: &Formatter<'_, 'a>,
) -> bool {
    let list = &parameters.items();
    if list.len() != 1 || parameters.rest.is_some() {
        return false;
    }

    // Safe because of the length check above
    let only_parameter = list.first().unwrap();

    if only_parameter.has_modifier() {
        return false;
    }

    // `(/* comment before */ only_parameter /* comment after */)`
    // Checker whether there are comments around the only parameter.
    if f.comments().has_comments_between(parameters.span.start, only_parameter.span.start)
        || f.comments().has_comments_between(only_parameter.span.end, parameters.span.end)
    {
        return false;
    }

    match &only_parameter.pattern.kind {
        BindingPatternKind::AssignmentPattern(assignment) => {
            assignment.left.kind.is_destructuring_pattern()
                && match &assignment.right {
                    Expression::ObjectExpression(object) => object.properties.is_empty(),
                    Expression::ArrayExpression(array) => array.elements.is_empty(),
                    Expression::Identifier(_) => true,
                    _ => false,
                }
        }
        BindingPatternKind::ArrayPattern(_) | BindingPatternKind::ObjectPattern(_) => true,
        BindingPatternKind::BindingIdentifier(_) => {
            parentheses_not_needed
                || only_parameter.pattern.type_annotation.as_ref().is_some_and(|ann| {
                    matches!(&ann.type_annotation, TSType::TSTypeLiteral(literal))
                })
        }
    }
}

/// Tests if all of the parameters of `expression` are simple enough to allow
/// a function to group.
pub fn has_only_simple_parameters(
    parameters: &FormalParameters<'_>,
    allow_type_annotations: bool,
) -> bool {
    parameters.items.iter().all(|parameter| is_simple_parameter(parameter, allow_type_annotations))
}

/// Tests if the single parameter is "simple", as in a plain identifier with no
/// explicit type annotation and no initializer:
///
/// Examples:
/// foo             => true
/// foo?            => true
/// foo = 'bar'     => false
/// foo: string     => false
/// {a, b}          => false
/// {a, b} = {}     => false
/// [a, b]          => false
///
fn is_simple_parameter(parameter: &FormalParameter<'_>, allow_type_annotations: bool) -> bool {
    parameter.pattern.kind.is_binding_identifier()
        && (allow_type_annotations || parameter.pattern.type_annotation.is_none())
}
