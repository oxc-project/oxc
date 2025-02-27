use oxc_allocator::Vec;
use oxc_ast::{AstKind, ast::*};
use oxc_span::{GetSpan, Span};

use crate::{
    Format, Prettier, array,
    format::print::{function_parameters, misc},
    group, hardline, if_break, indent,
    ir::Doc,
    line, softline, text,
};

pub enum ObjectLike<'a, 'b> {
    ObjectExpression(&'b ObjectExpression<'a>),
    ObjectPattern(&'b ObjectPattern<'a>),
    ObjectAssignmentTarget(&'b ObjectAssignmentTarget<'a>),
    TSTypeLiteral(&'b TSTypeLiteral<'a>),
    TSInterfaceBody(&'b TSInterfaceBody<'a>),
    TSEnumDeclaration(&'b TSEnumDeclaration<'a>),
}

impl<'a> ObjectLike<'a, '_> {
    /// This includes rest element for `ObjectPattern` and `ObjectAssignmentTarget`
    fn total_len(&self) -> usize {
        match self {
            ObjectLike::ObjectExpression(obj) => obj.properties.len(),
            ObjectLike::ObjectPattern(obj) => {
                obj.properties.len() + usize::from(obj.rest.is_some())
            }
            ObjectLike::ObjectAssignmentTarget(obj) => {
                obj.properties.len() + usize::from(obj.rest.is_some())
            }
            ObjectLike::TSTypeLiteral(obj) => obj.members.len(),
            ObjectLike::TSInterfaceBody(obj) => obj.body.len(),
            ObjectLike::TSEnumDeclaration(obj) => obj.members.len(),
        }
    }

    fn is_ts_type(&self) -> bool {
        matches!(self, ObjectLike::TSTypeLiteral(_) | ObjectLike::TSInterfaceBody(_))
    }

    fn separator(&self, p: &Prettier) -> Doc<'a> {
        if self.is_ts_type() {
            let semi = if p.options.semi { ";" } else { "" };
            return if_break!(p, text!(semi), text!(";"), None);
        }
        text!(",")
    }

    fn should_break(&self, p: &mut Prettier) -> bool {
        match self {
            ObjectLike::ObjectExpression(obj_expr) => {
                p.options.object_wrap.is_preserve()
                    && obj_expr.properties.first().is_some_and(|first_prop| {
                        misc::has_new_line_in_range(
                            p.source_text,
                            obj_expr.span.start,
                            first_prop.span().start,
                        )
                    })
            }
            ObjectLike::TSTypeLiteral(obj) => {
                p.options.object_wrap.is_preserve()
                    && obj.members.first().is_some_and(|first_prop| {
                        misc::has_new_line_in_range(
                            p.source_text,
                            obj.span.start,
                            first_prop.span().start,
                        )
                    })
            }
            ObjectLike::ObjectPattern(obj_pattern) => {
                let parent_kind = p.parent_kind();
                // `f(a, { <- THIS -> })` should not break
                !matches!(parent_kind, AstKind::FormalParameter(_))
                    && obj_pattern.properties.iter().any(|prop| {
                        matches!(
                            prop.value.kind,
                            BindingPatternKind::ObjectPattern(_)
                                | BindingPatternKind::ArrayPattern(_)
                        )
                    })
            }
            ObjectLike::ObjectAssignmentTarget(obj_target) => false,
            ObjectLike::TSInterfaceBody(_) | ObjectLike::TSEnumDeclaration(_) => true,
        }
    }

    fn print_properties(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut props = Vec::new_in(p.allocator);

        // Separator is determined by the previous property
        let mut separator_parts = array!(p, []);
        let next_separator_fn =
            |p: &Prettier<'a>, ts_signature: Option<&TSSignature>, span: Span| match (
                ts_signature.is_some_and(|ts_signature| {
                    // TODO: has_comment(ts_signature, CommentCheckFlags::PrettierIgnore),
                    let type_has_comment = false;
                    matches!(
                        ts_signature,
                        TSSignature::TSPropertySignature(_)
                            | TSSignature::TSMethodSignature(_)
                            | TSSignature::TSConstructSignatureDeclaration(_)
                            | TSSignature::TSCallSignatureDeclaration(_)
                    ) && type_has_comment
                }),
                p.is_next_line_empty(span),
            ) {
                (true, true) => array!(p, [line!(), hardline!(p)]),
                (true, false) => array!(p, [line!()]),
                (false, true) => array!(p, [self.separator(p), line!(), hardline!(p)]),
                (false, false) => array!(p, [self.separator(p), line!()]),
            };

        match self {
            ObjectLike::ObjectExpression(obj_expr) => {
                for prop in &obj_expr.properties {
                    props.push(std::mem::replace(
                        &mut separator_parts,
                        next_separator_fn(p, None, prop.span()),
                    ));
                    props.push(group!(p, [prop.format(p)]));
                }
            }
            ObjectLike::ObjectPattern(obj_pattern) => {
                for prop in &obj_pattern.properties {
                    props.push(std::mem::replace(
                        &mut separator_parts,
                        next_separator_fn(p, None, prop.span()),
                    ));
                    props.push(group!(p, [prop.format(p)]));
                }
                if let Some(rest) = &obj_pattern.rest {
                    props.push(std::mem::replace(
                        &mut separator_parts,
                        next_separator_fn(p, None, rest.span()),
                    ));
                    props.push(group!(p, [rest.format(p)]));
                }
            }
            ObjectLike::ObjectAssignmentTarget(obj_target) => {
                for prop in &obj_target.properties {
                    props.push(std::mem::replace(
                        &mut separator_parts,
                        next_separator_fn(p, None, prop.span()),
                    ));
                    props.push(group!(p, [prop.format(p)]));
                }
                if let Some(rest) = &obj_target.rest {
                    props.push(std::mem::replace(
                        &mut separator_parts,
                        next_separator_fn(p, None, rest.span()),
                    ));
                    props.push(group!(p, [rest.format(p)]));
                }
            }
            ObjectLike::TSTypeLiteral(obj) => {
                for member in &obj.members {
                    props.push(std::mem::replace(
                        &mut separator_parts,
                        next_separator_fn(p, Some(member), member.span()),
                    ));
                    props.push(group!(p, [member.format(p)]));
                }
            }
            ObjectLike::TSInterfaceBody(obj) => {
                for member in &obj.body {
                    props.push(std::mem::replace(
                        &mut separator_parts,
                        next_separator_fn(p, Some(member), member.span()),
                    ));
                    props.push(group!(p, [member.format(p)]));
                }
            }
            ObjectLike::TSEnumDeclaration(obj) => {
                for member in &obj.members {
                    props.push(std::mem::replace(
                        &mut separator_parts,
                        next_separator_fn(p, None, member.span()),
                    ));
                    props.push(group!(p, [member.format(p)]));
                }
            }
        }

        array!(p, props)
    }

    fn can_have_trailing_separator(&self) -> bool {
        debug_assert!(0 < self.total_len());

        #[expect(clippy::match_same_arms)]
        match self {
            ObjectLike::ObjectExpression(_) => true,
            ObjectLike::ObjectPattern(obj) => {
                if obj.rest.is_some() {
                    return false;
                }
                true
            }
            ObjectLike::ObjectAssignmentTarget(obj) => {
                if obj.rest.is_some() {
                    return false;
                }
                true
            }
            ObjectLike::TSTypeLiteral(obj) => {
                // TODO: && has_comment(last, CommentCheckFlags::PrettierIgnore)
                let last_has_comment = false;
                if obj.members.last().is_some_and(|last| {
                    matches!(
                        last,
                        TSSignature::TSPropertySignature(_)
                            | TSSignature::TSCallSignatureDeclaration(_)
                            | TSSignature::TSMethodSignature(_)
                            | TSSignature::TSConstructSignatureDeclaration(_)
                    ) && last_has_comment
                }) {
                    return false;
                }
                true
            }
            ObjectLike::TSInterfaceBody(obj) => {
                if obj.body.last().is_some_and(|last| {
                    // TODO: && has_comment(last, CommentCheckFlags::PrettierIgnore)
                    let last_has_comment = false;
                    matches!(
                        last,
                        TSSignature::TSPropertySignature(_)
                            | TSSignature::TSCallSignatureDeclaration(_)
                            | TSSignature::TSMethodSignature(_)
                            | TSSignature::TSConstructSignatureDeclaration(_)
                    ) && last_has_comment
                }) {
                    return false;
                }
                true
            }
            ObjectLike::TSEnumDeclaration(_) => true,
        }
    }
}

pub fn print_object<'a>(p: &mut Prettier<'a>, obj: &ObjectLike<'a, '_>) -> Doc<'a> {
    if obj.total_len() == 0 {
        // TODO: Comments
        // group!(p, [text!("{"), dangling_comment, softline!(), text!("}")])
        return text!("{}");
    }

    let bracket_spacing_fn = |p: &Prettier<'a>| {
        if p.options.bracket_spacing {
            return line!();
        }
        softline!()
    };

    let mut parts = Vec::new_in(p.allocator);

    parts.push(text!("{"));

    parts.push(indent!(p, [bracket_spacing_fn(p), obj.print_properties(p)]));
    if obj.can_have_trailing_separator()
        && (obj.is_ts_type() || p.options.trailing_comma.should_print_es5())
    {
        parts.push(if_break!(p, obj.separator(p)));
    }
    parts.push(bracket_spacing_fn(p));

    parts.push(text!("}"));

    let parent_kind = p.parent_kind();
    let should_break = obj.should_break(p);

    // If we inline the object as first argument of the parent,
    // we don't want to create another group so that the object breaks before the return type
    if matches!(obj, ObjectLike::ObjectPattern(obj_pattern))
        && match parent_kind {
            AstKind::FormalParameter(param) => param.decorators.is_empty(),
            _ => true,
        }
        && should_hug_the_only_parameter(p, parent_kind)
    {
        return array!(p, parts);
    }
    if matches!(obj, ObjectLike::TSTypeLiteral(_))
        && parent_kind.is_type()
        && p.parent_parent_kind().is_some_and(AstKind::is_type)
        && p.parent_parent_parent_kind().is_some_and(|k| should_hug_the_only_parameter(p, k))
    {
        return array!(p, parts);
    }
    // Assignment printing logic (printAssignment) is responsible for adding a group if needed
    if !should_break
        && matches!(obj, ObjectLike::ObjectPattern(_))
        && matches!(parent_kind, AstKind::AssignmentExpression(_) | AstKind::VariableDeclarator(_))
    {
        return array!(p, parts);
    }

    group!(p, parts, should_break, None)
}

fn should_hug_the_only_parameter(p: &Prettier<'_>, kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::FormalParameters(params) => {
            function_parameters::should_hug_the_only_function_parameter(p, params)
        }
        _ => false,
    }
}
