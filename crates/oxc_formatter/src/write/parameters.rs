use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodeIterator, AstNodes},
    format_args,
    formatter::{
        Format, FormatResult, Formatter, prelude::*, separated::FormatSeparatedIter,
        trivia::FormatTrailingComments,
    },
    options::{FormatTrailingCommas, TrailingSeparator},
    utils::call_expression::is_test_call_expression,
    write,
};

use super::FormatWrite;

pub fn get_this_param<'a>(parent: &AstNodes<'a>) -> Option<&'a AstNode<'a, TSThisParameter<'a>>> {
    match parent {
        AstNodes::Function(func) => func.this_param(),
        AstNodes::TSFunctionType(func) => func.this_param(),
        AstNodes::TSMethodSignature(func) => func.this_param(),
        AstNodes::TSCallSignatureDeclaration(func) => func.this_param(),
        _ => None,
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, FormalParameters<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        // `function foo /**/ () {}`
        //               ^^^ keep comments printed before parameters
        let comments = f.context().comments().comments_before(self.span.start);
        if !comments.is_empty() {
            write!(f, [space(), FormatTrailingComments::Comments(comments)])?;
        }

        let parentheses_not_needed = if let AstNodes::ArrowFunctionExpression(arrow) = self.parent {
            can_avoid_parentheses(arrow, f)
        } else {
            false
        };

        let this_param = get_this_param(self.parent);

        let has_any_decorated_parameter =
            self.items.iter().any(|param| !param.decorators.is_empty());

        let can_hug = should_hug_function_parameters(self, this_param, parentheses_not_needed, f)
            && !has_any_decorated_parameter;

        let layout = if !self.has_parameter() && this_param.is_none() {
            ParameterLayout::NoParameters
        } else if can_hug || {
            // Check if these parameters are part of a test call expression
            // by walking up the parent chain
            let mut current_parent = Some(self.parent);
            let mut is_in_test_call = false;

            while let Some(parent) = current_parent {
                // Stop at root (Dummy node provides natural termination)
                if matches!(parent, AstNodes::Dummy()) {
                    break;
                }

                if let AstNodes::CallExpression(call) = parent
                    && is_test_call_expression(call)
                {
                    is_in_test_call = true;
                    break;
                }

                current_parent = Some(parent.parent());
            }

            is_in_test_call
        } {
            ParameterLayout::Hug
        } else {
            ParameterLayout::Default
        };

        if !parentheses_not_needed {
            write!(f, "(")?;
        }

        match layout {
            ParameterLayout::NoParameters => {
                write!(f, format_dangling_comments(self.span()).with_soft_block_indent())?;
            }
            ParameterLayout::Hug => {
                write!(f, ParameterList::with_layout(self, this_param, layout))?;
            }
            ParameterLayout::Default => {
                write!(
                    f,
                    soft_block_indent(&format_args!(&ParameterList::with_layout(
                        self, this_param, layout
                    )))
                );
            }
        }

        if !parentheses_not_needed {
            write!(f, [")"])?;
        }

        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, FormalParameter<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let content = format_with(|f| {
            if let Some(accessibility) = self.accessibility() {
                write!(f, [accessibility.as_str(), space()])?;
            }
            if self.r#override() {
                write!(f, ["override", space()])?;
            }
            if self.readonly() {
                write!(f, ["readonly", space()])?;
            }
            write!(f, self.pattern())
        });

        let is_hug_parameter = matches!(self.parent, AstNodes::FormalParameters(params) if {
            let (parentheses_not_needed, this_param) = if let AstNodes::ArrowFunctionExpression(arrow) = params.parent {
                (can_avoid_parentheses(arrow, f), None)
            } else {
                (false, get_this_param(params.parent))
            };
            should_hug_function_parameters(params, this_param, parentheses_not_needed, f)
        });

        let decorators = self.decorators();

        if is_hug_parameter && decorators.is_empty() {
            write!(f, [&content])
        } else if decorators.is_empty() {
            write!(f, [group(&content)])
        } else {
            write!(f, [group(&decorators), group(&content)])
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSThisParameter<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["this", self.type_annotation()])
    }
}

enum Parameter<'a, 'b> {
    This(&'b AstNode<'a, TSThisParameter<'a>>),
    FormalParameter(&'b AstNode<'a, FormalParameter<'a>>),
    Rest(&'b AstNode<'a, BindingRestElement<'a>>),
}

impl GetSpan for Parameter<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Self::This(param) => param.span(),
            Self::FormalParameter(param) => param.span(),
            Self::Rest(e) => e.span(),
        }
    }
}

impl<'a> Format<'a> for Parameter<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            Self::This(param) => param.fmt(f),
            Self::FormalParameter(param) => param.fmt(f),
            Self::Rest(e) => e.fmt(f),
        }
    }
}

struct FormalParametersIter<'a, 'b> {
    this: Option<&'b AstNode<'a, TSThisParameter<'a>>>,
    params: AstNodeIterator<'a, FormalParameter<'a>>,
    rest: Option<&'b AstNode<'a, BindingRestElement<'a>>>,
}

impl<'a, 'b> From<&'b ParameterList<'a, 'b>> for FormalParametersIter<'a, 'b> {
    fn from(value: &'b ParameterList<'a, 'b>) -> Self {
        Self { this: value.this, params: value.list.items().iter(), rest: value.list.rest() }
    }
}

impl<'a, 'b> Iterator for FormalParametersIter<'a, 'b> {
    type Item = Parameter<'a, 'b>;

    fn next(&mut self) -> Option<Self::Item> {
        self.this.take().map(Parameter::This).or_else(|| {
            self.params
                .next()
                .map(Parameter::FormalParameter)
                .or_else(|| self.rest.take().map(Parameter::Rest))
        })
    }
}

pub struct ParameterList<'a, 'b> {
    list: &'b AstNode<'a, FormalParameters<'a>>,
    this: Option<&'b AstNode<'a, TSThisParameter<'a>>>,
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
        this: Option<&'b AstNode<'a, TSThisParameter<'a>>>,
        layout: ParameterLayout,
    ) -> Self {
        Self { list, this, layout: Some(layout) }
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
                joiner
                    .entries_with_trailing_separator(
                        FormalParametersIter::from(self),
                        ",",
                        trailing_separator,
                    )
                    .finish()
            }
            Some(ParameterLayout::Hug) => {
                let mut join = f.join_with(space());
                join.entries_with_trailing_separator(
                    FormalParametersIter::from(self),
                    ",",
                    TrailingSeparator::Omit,
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
        && !f.comments().has_comment_in_span(arrow.params.span)
}

pub fn should_hug_function_parameters<'a>(
    parameters: &AstNode<'a, FormalParameters<'a>>,
    this_param: Option<&AstNode<'a, TSThisParameter<'a>>>,
    parentheses_not_needed: bool,
    f: &Formatter<'_, 'a>,
) -> bool {
    let list = &parameters.items();

    if list.len() > 1 || parameters.rest.is_some() {
        return false;
    }

    if let Some(this_param) = this_param {
        // `(/* comment before */ this /* comment after */)`
        // Checker whether there are comments around the only parameter.

        if f.comments().has_comment_in_range(parameters.span.start, this_param.span.start)
            || f.comments().has_comment_in_range(this_param.span.end, parameters.span.end)
        {
            return false;
        }

        return list.is_empty()
            && this_param
                .type_annotation
                .as_ref()
                .is_none_or(|ty| matches!(ty.type_annotation, TSType::TSTypeLiteral(_)));
    }

    // Safe because of the length check above
    let Some(only_parameter) = list.first() else { return false };

    if only_parameter.has_modifier() {
        return false;
    }

    // `(/* comment before */ only_parameter /* comment after */)`
    // Checker whether there are comments around the only parameter.
    if f.comments().has_comment_in_range(parameters.span.start, only_parameter.span.start)
        || f.comments().has_comment_in_range(only_parameter.span.end, parameters.span.end)
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
                    matches!(
                        &ann.type_annotation,
                        TSType::TSTypeLiteral(_) | TSType::TSMappedType(_)
                    )
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
