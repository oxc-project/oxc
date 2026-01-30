use std::ops::Deref;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Semicolons, TrailingSeparator,
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        Buffer, Formatter,
        prelude::*,
        separated::FormatSeparatedIter,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    parentheses::NeedsParentheses,
    print::{function::should_group_function_parameters, semicolon::OptionalSemicolon},
    utils::{
        assignment_like::AssignmentLike,
        format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
        object::{format_property_key, should_preserve_quote},
    },
    write,
};

use super::{
    FormatWrite,
    type_parameters::{FormatTSTypeParameters, FormatTSTypeParametersOptions},
};

impl<'a> FormatWrite<'a> for AstNode<'a, ClassBody<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        if f.options().quote_properties.is_consistent() {
            let quote_needed = self.body.iter().any(|signature| {
                let key = match signature {
                    ClassElement::PropertyDefinition(property) => &property.key,
                    ClassElement::AccessorProperty(property) => &property.key,
                    ClassElement::MethodDefinition(method) => &method.key,
                    _ => return false,
                };

                should_preserve_quote(key, f)
            });
            f.context_mut().push_quote_needed(quote_needed);
        }

        write!(f, ["{", block_indent(&self.body()), "}"]);

        if f.options().quote_properties.is_consistent() {
            f.context_mut().pop_quote_needed();
        }
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, ClassElement<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        // Join class elements with hard line breaks between them
        let mut join = f.join_nodes_with_hardline();
        // Iterate through pairs of consecutive elements to handle semicolons properly
        // Each element is paired with the next one (or None for the last element)
        let mut iter = self.iter().peekable();
        while let Some(element) = iter.next() {
            join.entry(element.span(), &(element, iter.peek().copied()));
        }
    }
}

impl<'a> Format<'a> for (&AstNode<'a, ClassElement<'a>>, Option<&AstNode<'a, ClassElement<'a>>>) {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        FormatClassElementWithSemicolon::new(self.0, self.1).fmt(f);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, MethodDefinition<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, [self.decorators()]);

        if let Some(accessibility) = &self.accessibility {
            write!(f, [accessibility.as_str(), space()]);
        }
        if self.r#static {
            write!(f, ["static", space()]);
        }
        if self.r#type.is_abstract() {
            write!(f, ["abstract", space()]);
        }
        if self.r#override {
            write!(f, ["override", space()]);
        }
        match &self.kind {
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Method => {}
            MethodDefinitionKind::Get => {
                write!(f, ["get", space()]);
            }
            MethodDefinitionKind::Set => {
                write!(f, ["set", space()]);
            }
        }
        let value = self.value();

        if value.r#async {
            write!(f, ["async", space()]);
        }
        if value.generator {
            write!(f, "*");
        }
        if self.computed {
            write!(f, ["[", self.key(), "]"]);
        } else {
            format_property_key(self.key(), f);
        }

        if self.optional {
            write!(f, "?");
        }

        format_grouped_parameters_with_return_type_for_method(
            value.type_parameters(),
            value.this_param.as_deref(),
            value.params(),
            value.return_type(),
            f,
        );

        if let Some(body) = &value.body() {
            write!(f, body);
        } else {
            let comments = f.context().comments().comments_before(self.span.end);
            write!(f, FormatTrailingComments::Comments(comments));
        }

        if self.r#type().is_abstract()
            || matches!(value.r#type, FunctionType::TSEmptyBodyFunctionExpression)
        {
            write!(f, OptionalSemicolon);
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, PropertyDefinition<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        AssignmentLike::PropertyDefinition(self).fmt(f);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, PrivateIdentifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["#", text_without_whitespace(self.name().as_str())]);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, StaticBlock<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["static", space(), "{"]);

        if self.body.is_empty() {
            write!(f, [format_dangling_comments(self.span).with_block_indent()]);
        } else {
            write!(f, [block_indent(&self.body())]);
        }

        write!(f, "}");
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, AccessorProperty<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, [self.decorators()]);

        if let Some(accessibility) = self.accessibility() {
            write!(f, [accessibility.as_str(), space()]);
        }
        if self.r#static {
            write!(f, ["static", space()]);
        }
        if self.r#type.is_abstract() {
            write!(f, ["abstract", space()]);
        }
        if self.r#override {
            write!(f, ["override", space()]);
        }
        write!(f, ["accessor", space()]);
        if self.computed {
            write!(f, "[");
        }
        write!(f, self.key());
        if self.computed {
            write!(f, "]");
        }
        if let Some(type_annotation) = &self.type_annotation() {
            write!(f, type_annotation);
        }
        if let Some(value) = &self.value() {
            write!(f, [space(), "=", space(), value]);
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSIndexSignature<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        if self.r#static {
            write!(f, ["static", space()]);
        }
        if self.readonly {
            write!(f, ["readonly", space()]);
        }
        let is_class = matches!(self.parent(), AstNodes::ClassBody(_));
        write!(
            f,
            [
                "[",
                self.parameters(),
                "]",
                self.type_annotation(),
                is_class.then_some(OptionalSemicolon)
            ]
        );
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSIndexSignatureName<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        f.join_with(&soft_line_break_or_space()).entries_with_trailing_separator(
            self.iter(),
            ",",
            TrailingSeparator::Disallowed,
        );
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSIndexSignatureName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, [text_without_whitespace(self.name().as_str()), self.type_annotation()]);
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, TSClassImplements<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let last_index = self.len().saturating_sub(1);
        let mut joiner = f.join_with(soft_line_break_or_space());

        for (i, heritage) in FormatSeparatedIter::new(self.into_iter(), ",")
            .with_trailing_separator(TrailingSeparator::Disallowed)
            .enumerate()
        {
            if i == last_index {
                // The trailing comments of the last heritage should be printed inside the class declaration
                joiner.entry(&FormatNodeWithoutTrailingComments(&heritage));
            } else {
                joiner.entry(&heritage);
            }
        }
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSClassImplements<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, [self.expression(), self.type_arguments()]);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Class<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        if self.r#type == ClassType::ClassExpression
            && (!self.decorators.is_empty() && self.needs_parentheses(f))
        {
            write!(f, soft_block_indent(&FormatClass(self)));
        } else {
            FormatClass(self).fmt(f);
        }
    }
}

struct FormatClass<'a, 'b>(pub &'b AstNode<'a, Class<'a>>);

impl<'a> Deref for FormatClass<'a, '_> {
    type Target = AstNode<'a, Class<'a>>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> Format<'a> for FormatClass<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let decorators = self.decorators();
        let type_parameters = self.type_parameters();
        let super_class = self.super_class();
        let implements = self.implements();
        let body = self.body();

        // Decorators are handled differently depending on the parent context
        // When the class is exported, the export statement handles decorator formatting
        // to ensure proper placement relative to the export keyword
        if self.is_expression()
            || !matches!(
                self.parent(),
                AstNodes::ExportNamedDeclaration(_) | AstNodes::ExportDefaultDeclaration(_)
            )
        {
            write!(f, decorators);
        }

        if self.declare {
            write!(f, ["declare", space()]);
        }
        if self.r#abstract {
            write!(f, ["abstract", space()]);
        }

        write!(f, "class");

        let head = format_with(|f| {
            if let Some(id) = self.id() {
                write!(f, [space(), id]);
            }

            if let Some(type_parameters) = &type_parameters {
                let type_parameters_id = Some(f.group_id("type_parameters"));
                write!(
                    f,
                    FormatTSTypeParameters::new(
                        type_parameters,
                        FormatTSTypeParametersOptions {
                            group_id: type_parameters_id,
                            is_type_or_interface_decl: false
                        }
                    )
                );
                if super_class.is_some() || !implements.is_empty() {
                    type_parameters.format_trailing_comments(f);
                }
            }

            // Handle comments that appear between the class name and the extends keyword.
            // This is specifically for preserving comments in patterns like:
            //   class A // comment 1
            //     // comment 2
            //     extends B {}
            //
            // The logic here ensures these comments are formatted as trailing comments
            // after the class name, maintaining their position before the extends clause.
            if let Some(super_class) = &super_class {
                let comments = f.context().comments().comments_before(super_class.span().start);
                if comments.iter().any(|c| c.preceded_by_newline()) {
                    indent(&FormatTrailingComments::Comments(comments)).fmt(f);
                }
            }
        });

        let group_mode = should_group(self, f);

        let format_heritage_clauses = format_with(|f| {
            if let Some(extends) = super_class {
                // Format the extends clause with its expression and optional type arguments
                let format_super = format_with(|f| {
                    let type_arguments = self.super_type_arguments();

                    // Collect comments after the extends expression (and type arguments if present)
                    // These comments need careful handling to preserve their association
                    // with the extends clause rather than the class body
                    let comments = if type_arguments.is_some() || !implements.is_empty() {
                        &[]
                    } else {
                        f.context()
                            .comments()
                            .comments_in_range(extends.span().end, body.span.start)
                    };

                    // Check if there are trailing line comments after the extends clause
                    // These comments need special handling to ensure they're placed correctly
                    // relative to the extends expression and any type arguments
                    let has_trailing_comments = comments.iter().any(|comment| comment.is_line());

                    let content = format_with(|f| {
                        if let Some(type_arguments) = type_arguments {
                            write!(f, [extends]);
                            if implements.is_empty() {
                                type_arguments.write(f);
                            } else {
                                type_arguments.fmt(f);
                            }
                        } else if implements.is_empty() {
                            FormatNodeWithoutTrailingComments(extends).fmt(f);
                            // Only add trailing comments if they're not line comments
                            // Line comments are handled separately to ensure proper placement
                            if !has_trailing_comments {
                                FormatTrailingComments::Comments(comments).fmt(f);
                            }
                        } else {
                            extends.fmt(f);
                        }
                    });

                    if matches!(extends.grand_parent(), AstNodes::AssignmentExpression(_)) {
                        let content = content.memoized();
                        write!(
                            f,
                            [group(&format_args!(
                                &if_group_breaks(&format_args!(
                                    token("("),
                                    &soft_block_indent(&content),
                                    token(")"),
                                )),
                                &if_group_fits_on_line(&content)
                            ))]
                        );
                    } else {
                        content.fmt(f);
                    }
                });

                let format_extends =
                    format_with(|f| write!(f, [space(), "extends", space(), &format_super]));

                if group_mode {
                    write!(f, [soft_line_break_or_space(), group(&format_extends)]);
                } else {
                    write!(f, format_extends);
                }
            }

            if !implements.is_empty() {
                let leading_comments =
                    f.context().comments().comments_before(implements[0].span().start);

                if usize::from(super_class.is_some()) + implements.len() > 1 {
                    write!(
                        f,
                        [
                            soft_line_break_or_space(),
                            FormatLeadingComments::Comments(leading_comments),
                            (!leading_comments.is_empty()).then_some(hard_line_break()),
                            "implements",
                            group(&soft_line_indent_or_space(implements))
                        ]
                    );
                } else {
                    let format_inner = format_with(|f| {
                        write!(
                            f,
                            [
                                FormatLeadingComments::Comments(leading_comments),
                                "implements",
                                space(),
                                implements,
                            ]
                        );
                    });

                    if group_mode {
                        write!(f, [soft_line_break_or_space(), group(&format_inner)]);
                    } else {
                        write!(f, [space(), format_inner]);
                    }
                }
            }
        });

        if group_mode {
            let indented = format_with(|f| write!(f, [head, indent(&format_heritage_clauses)]));

            let heritage_id = f.group_id("heritageGroup");
            write!(f, [group(&indented).with_group_id(Some(heritage_id)), space()]);

            if !body.body.is_empty() {
                write!(f, [if_group_breaks(&hard_line_break()).with_group_id(Some(heritage_id))]);
            }
        } else {
            write!(f, [head, format_heritage_clauses, space()]);
        }

        let leading_comments = f.context().comments().comments_before(self.body.span.start);
        if leading_comments.iter().any(|c| !c.is_line()) {
            write!(f, FormatLeadingComments::Comments(leading_comments));
        }

        if body.body.is_empty() {
            write!(f, ["{", format_dangling_comments(self.span).with_block_indent(), "}"]);
        } else {
            body.fmt(f);
        }
    }
}

/// Determines whether class heritage clauses (extends/implements) should be grouped.
///
/// Grouping affects how line breaks are handled - grouped elements try to fit
/// on the same line but break together if they don't fit.
///
/// Heritage clauses are grouped when:
/// 1. Superclass and/or implements are more than one
/// 2. Superclass is a member expression and has no type arguments
///   - ClassExpression: its parent is not an AssignmentExpression
///   - ClassDeclaration: always
/// 3. Implements is a qualified name and has no type arguments
/// 4. There are comments in the heritage clause area
/// 5. There are trailing line comments after type parameters
fn should_group<'a>(class: &AstNode<Class<'a>>, f: &Formatter<'_, 'a>) -> bool {
    if usize::from(class.super_class.is_some()) + class.implements.len() > 1 {
        return true;
    }

    if (!class.is_expression() || !matches!(class.parent(), AstNodes::AssignmentExpression(_)))
        && class
            .super_class
            .as_ref()
            .is_some_and(|super_class|
                super_class.is_member_expression() ||
                matches!(&super_class, Expression::ChainExpression(chain) if chain.expression.is_member_expression())
            ) && class.super_type_arguments.is_none()
        || class.implements.first().is_some_and(|implements| {
            implements.type_arguments.is_none() && implements.expression.is_qualified_name()
        })
    {
        return true;
    }

    let comments = f.comments();

    let id_span = class.id.as_ref().map(GetSpan::span);
    let type_parameters_span = class.type_parameters.as_ref().map(|t| t.span);
    let super_class_span = class.super_class.as_ref().map(GetSpan::span);
    let super_type_arguments_span = class.super_type_arguments.as_ref().map(|t| t.span);
    let implements_span = class.implements.first().map(GetSpan::span);

    let spans = [
        id_span,
        type_parameters_span,
        super_class_span,
        super_type_arguments_span,
        implements_span,
    ];

    let mut spans_iter = spans.iter().flatten().peekable();

    while let Some(span) = spans_iter.next() {
        if let Some(next_span) = spans_iter.peek() {
            // Check if there are comments between the current span and the next one
            if comments.has_comment_in_range(span.end, next_span.start) {
                // If there are comments, we should group the heritage clauses
                return true;
            }
        } else {
            break;
        }
    }
    false
}

pub struct FormatClassElementWithSemicolon<'a, 'b> {
    element: &'b AstNode<'a, ClassElement<'a>>,
    next_element: Option<&'b AstNode<'a, ClassElement<'a>>>,
}

impl<'a, 'b> FormatClassElementWithSemicolon<'a, 'b> {
    pub fn new(
        element: &'b AstNode<'a, ClassElement<'a>>,
        next_element: Option<&'b AstNode<'a, ClassElement<'a>>>,
    ) -> Self {
        Self { element, next_element }
    }

    fn needs_semicolon(&self) -> bool {
        let Self { element, next_element, .. } = self;

        if let ClassElement::PropertyDefinition(def) = element.as_ref()
            && def.value.is_none()
            && def.type_annotation.is_none()
            && matches!(&def.key, PropertyKey::StaticIdentifier(ident) if matches!(ident.name.as_str(), "static" | "get" | "set") )
        {
            return true;
        }

        let Some(next_element) = next_element else { return false };

        match next_element.as_ref() {
            // When the name starts with the generator token or `[`
            ClassElement::MethodDefinition(def) if !def.value.r#async => {
                (def.computed
                    && !(def.kind.is_accessor()
                        || def.r#static
                        || def.accessibility.is_some()
                        || def.r#override))
                    || def.value.generator
            }
            ClassElement::PropertyDefinition(def) => {
                def.computed
                    && !(def.accessibility.is_some()
                        || def.r#static
                        || def.declare
                        || def.r#override
                        || def.readonly)
            }
            ClassElement::AccessorProperty(def) => {
                def.computed && !(def.accessibility.is_some() || def.r#static || def.r#override)
            }
            ClassElement::TSIndexSignature(_) => true,
            _ => false,
        }
    }
}

impl<'a> Format<'a> for FormatClassElementWithSemicolon<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let needs_semi = matches!(
            self.element.as_ref(),
            ClassElement::PropertyDefinition(_) | ClassElement::AccessorProperty(_)
        );

        let needs_semi = needs_semi
            && match f.options().semicolons {
                Semicolons::Always => true,
                Semicolons::AsNeeded => self.needs_semicolon(),
            }
            // Don't add semicolon if the element is suppressed (has `oxfmt-ignore`),
            // because the suppressed source text already includes the original semicolon.
            && !f.comments().is_suppressed(self.element.span().start);

        if needs_semi {
            write!(f, [FormatNodeWithoutTrailingComments(self.element), ";"]);
            // Print trailing comments after the semicolon
            match self.element.as_ast_nodes() {
                AstNodes::PropertyDefinition(prop) => {
                    prop.format_trailing_comments(f);
                }
                AstNodes::AccessorProperty(prop) => {
                    prop.format_trailing_comments(f);
                }
                _ => {
                    unreachable!("Only `PropertyDefinition` and `AccessorProperty` can reach here");
                }
            }
        } else {
            write!(f, self.element);
        }
    }
}

/// Based on <https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/print/function.js#L160-L176>
pub fn format_grouped_parameters_with_return_type_for_method<'a>(
    type_parameters: Option<&AstNode<'a, TSTypeParameterDeclaration<'a>>>,
    this_param: Option<&TSThisParameter<'a>>,
    params: &AstNode<'a, FormalParameters<'a>>,
    return_type: Option<&AstNode<'a, TSTypeAnnotation<'a>>>,
    f: &mut Formatter<'_, 'a>,
) {
    write!(f, type_parameters);

    group(&format_with(|f| {
        let format_parameters = params.memoized();
        let format_return_type = return_type.map(FormatNodeWithoutTrailingComments).memoized();

        // Inspect early, in case the `return_type` is formatted before `parameters`
        // in `should_group_function_parameters`.
        format_parameters.inspect(f);

        let should_break_parameters = should_break_function_parameters(params);
        let should_group_parameters = should_break_parameters
            || should_group_function_parameters(
                type_parameters.map(AsRef::as_ref),
                params.parameters_count() + usize::from(this_param.is_some()),
                return_type.map(AsRef::as_ref),
                &format_return_type,
                f,
            );

        if should_group_parameters {
            write!(f, [group(&format_parameters).should_expand(should_break_parameters)]);
        } else {
            write!(f, [format_parameters]);
        }

        write!(f, [format_return_type]);
    }))
    .fmt(f);
}

/// Decide if a constructor parameter list should prefer a
/// multi-line layout.
///
/// If there is more than one parameter, and any parameter has a modifier
///   (e.g. a TypeScript parameter property like `public/private/protected`, or
///   `readonly`, etc.), we break the parameters onto multiple lines.
//
/// Examples
/// --------
/// Multiple params with a modifier → break:
///
/// ```ts
/// // input
/// constructor(public x: number, y: number) {}
///
/// // preferred layout
/// constructor(
///   public x: number,
///   y: number,
/// ) {}
/// ```
///
/// Single param with a modifier → keep inline:
///
/// ```ts
/// constructor(private id: string) {}
/// ```
fn should_break_function_parameters<'a>(params: &AstNode<'a, FormalParameters<'a>>) -> bool {
    params.parameters_count() > 1 && params.items().iter().any(|param| param.has_modifier())
}
