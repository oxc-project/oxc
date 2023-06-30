#![allow(clippy::too_many_lines)]
#![allow(clippy::unimplemented)]
use std::{collections::BTreeMap, rc::Rc, sync::Arc};

use oxc_ast::{
    ast::{
        Argument, ArrayPattern, AssignmentExpression, AssignmentPattern, AssignmentTarget,
        AssignmentTargetPattern, BindingPatternKind, Expression, FormalParameters,
        ImportDeclarationSpecifier, ImportOrExportKind, JSXClosingElement, JSXElementName,
        JSXOpeningElement, ModuleDeclaration, ObjectPattern, ObjectPropertyKind, PropertyKey,
        SimpleAssignmentTarget, StringLiteral, TSLiteral, TSSignature, TSType, TSTypeAnnotation,
        TSTypeName, TemplateLiteral, VariableDeclarator,
    },
    AstKind,
};
use oxc_semantic::{AstNode, Semantic};
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;
use serde::Deserialize;
use trustfall::{
    provider::{
        resolve_coercion_with, resolve_neighbors_with, resolve_property_with, Adapter,
        ContextIterator, ContextOutcomeIterator, EdgeParameters, ResolveEdgeInfo, ResolveInfo,
        TrustfallEnumVertex, VertexIterator,
    },
    FieldValue,
};

#[derive(Debug, Clone, Deserialize)]
pub struct InputQuery {
    pub query: String,
    pub args: BTreeMap<Arc<str>, FieldValue>,
    pub reason: String,
}

#[derive(Debug, Clone, TrustfallEnumVertex)]
pub enum Vertex<'a> {
    AstNode(AstNode<'a>),
    Expression(&'a Expression<'a>),
    Argument(&'a Argument<'a>, i32), // i32 is argument index
    Span(Span),
    Parameter(&'a BindingPatternKind<'a>, i32), // i32 is argument index
    PossiblyNumber(f64),
    PropertyAccess(Span, &'a str),
    PropertyKey(&'a PropertyKey<'a>),
    TSType(&'a TSType<'a>),
    TSSignature(&'a TSSignature<'a>),
    TSTypeAnnotation(&'a TSTypeAnnotation<'a>),
    TSLiteral(&'a TSLiteral<'a>, Span),
    TSTypeName(&'a TSTypeName<'a>, Span),
    EffectivelyLiteralString(Span, &'a Atom),
    ImportSpecifier(&'a ImportDeclarationSpecifier),
    JsxElementName(&'a JSXElementName<'a>),
    JsxOpeningElement(&'a JSXOpeningElement<'a>),
    JsxClosingElement(&'a JSXClosingElement<'a>),
    ObjectProperty(&'a ObjectPropertyKind<'a>),
    VariableDeclaration(&'a VariableDeclarator<'a>),
    // operators
    Operator(&'a BinaryOperator),
    AssignmentOperator(&'a AssignmentExpression<'a>),
    // assignment to ...
    AssignmentToIdentifier(Span, &'a Atom),
    AssignmentOnObject(&'a SimpleAssignmentTarget<'a>),
    DestructuringAssignment(DestructuringAssignment<'a>),
    File,
}

#[derive(Debug, Clone)]
pub enum DestructuringAssignment<'a> {
    AssignmentExpression(&'a AssignmentTargetPattern<'a>),
    ArrayPattern(&'a ArrayPattern<'a>),
    ObjectPattern(&'a ObjectPattern<'a>),
    Defaulted(&'a AssignmentPattern<'a>),
}

impl<'a> GetSpan for DestructuringAssignment<'a> {
    fn span(&self) -> Span {
        match self {
            DestructuringAssignment::AssignmentExpression(expr) => expr.span(),
            DestructuringAssignment::ArrayPattern(expr) => expr.span,
            DestructuringAssignment::ObjectPattern(expr) => expr.span,
            DestructuringAssignment::Defaulted(expr) => expr.span,
        }
    }
}

pub struct LintAdapter<'a> {
    pub semantic: Rc<Semantic<'a>>,
}

macro_rules! get_ast_or_x {
    ($v:ident, $alternate_vertex:ident, $typed:ident) => {
        match $v {
            Vertex::AstNode(node) => match node.kind() {
                AstKind::$typed(be) => be,
                _ => unreachable!(),
            },
            Vertex::$alternate_vertex($alternate_vertex::$typed(be)) => &be.0,
            _ => unreachable!(),
        }
    };
}

macro_rules! coerce_ast_or_x_match {
    ($v:ident, $alternate_vertex:ident, $typed:ident) => {
        match $v {
            Vertex::AstNode(node) => matches!(node.kind(), AstKind::$typed(..)),
            Vertex::$alternate_vertex(expr) => matches!(expr, $alternate_vertex::$typed(..)),
            _ => false,
        }
    };
}

macro_rules! coerce_ast_or_x {
    ($contexts:ident, $alternate_vertex:ident, $typed:ident) => {
        resolve_coercion_with($contexts, |v| match v {
            Vertex::AstNode(node) => matches!(node.kind(), AstKind::$typed(..)),
            Vertex::$alternate_vertex(expr) => matches!(expr, $alternate_vertex::$typed(..)),
            _ => false,
        })
    };
}

macro_rules! coerce_ast {
    ($contexts:ident, $typed:ident) => {
        resolve_coercion_with($contexts, |v| match v {
            Vertex::AstNode(node) => matches!(node.kind(), AstKind::$typed(..)),
            _ => false,
        })
    };
}

macro_rules! get_ast_or_x_or_y {
    ($v:ident, $x:ident, $y:ident, $typed:ident) => {
        match $v {
            Vertex::AstNode(node) => match node.kind() {
                AstKind::$typed(be) => be,
                _ => unreachable!(),
            },
            Vertex::$x($x::$typed(be), ..) => &be.0,
            Vertex::$y($y::$typed(be), ..) => &be.0,
            _ => unreachable!(),
        }
    };
}

macro_rules! coerce_ast_or_x_or_y {
    ($contexts:ident, $alternate_x:ident, $alternate_y:ident, $typed:ident) => {
        resolve_coercion_with($contexts, |v| match v {
            Vertex::AstNode(node) => matches!(node.kind(), AstKind::$typed(..)),
            Vertex::$alternate_x(expr, ..) => matches!(expr, $alternate_x::$typed(..)),
            Vertex::$alternate_y(expr, ..) => matches!(expr, $alternate_y::$typed(..)),
            _ => unreachable!(),
        })
    };
}

macro_rules! string_matcher {
    ($Self:ident, $t:ident, $expr:expr) => {
        match $expr {
            $t::StringLiteral(slit) => slit.value.as_str().into(),
            $t::TemplateLiteral(tlit) => $Self::field_value_from_template_lit(tlit),
            _ => FieldValue::Null,
        }
    };
}

macro_rules! ast_node_or_expression_field {
    ($self:ident, $typed:ident, $attr:ident) => {
        match $self {
            Vertex::AstNode(node) => match node.kind() {
                AstKind::$typed(be) => &be.$attr,
                _ => unreachable!(),
            },
            Vertex::Expression(Expression::$typed(be)) => &be.$attr,
            _ => unreachable!(),
        }
    };
}

macro_rules! once_vertex_iter_if_some {
    ($maybe_some:expr, $vertex_type:ident) => {
        if let Some(some) = &$maybe_some {
            Box::new(std::iter::once(Vertex::$vertex_type(&some.0)))
        } else {
            Box::new(std::iter::empty())
        }
    };
}

fn solve_member_expr_for_lowest<'a>(object: &'a Expression<'a>) -> &'a Expression<'a> {
    let mut obj = object;

    while let Expression::MemberExpression(b) = obj {
        obj = b.object();
    }

    obj
}

fn formal_parameters_to_iter<'a>(
    params: &'a FormalParameters<'a>,
) -> VertexIterator<'a, Vertex<'a>> {
    Box::new(
        params
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| Vertex::Parameter(&item.pattern.kind, index.try_into().unwrap())),
    )
}

fn field_value_from_template_lit(tlit: &TemplateLiteral) -> FieldValue {
    if tlit.expressions.len() == 0 && tlit.quasis.len() == 1 {
        let quasi = &tlit.quasis[0].value;
        FieldValue::String(quasi.cooked.as_ref().unwrap_or(&quasi.raw).as_str().into())
    } else {
        FieldValue::Null
    }
}

fn assignment_target_to_vertex<'a>(at: &'a AssignmentTarget<'a>) -> Vertex<'a> {
    match &at {
        AssignmentTarget::SimpleAssignmentTarget(simple_assignment_target) => {
            match simple_assignment_target {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                    Vertex::AssignmentToIdentifier(ident.span, &ident.name)
                }
                _ => Vertex::AssignmentOnObject(simple_assignment_target),
            }
        }
        AssignmentTarget::AssignmentTargetPattern(d) => {
            Vertex::DestructuringAssignment(DestructuringAssignment::AssignmentExpression(d))
        }
    }
}

fn binding_pattern_kind_to_vertex<'a>(bpk: &'a BindingPatternKind<'a>) -> Vertex<'a> {
    match bpk {
        BindingPatternKind::BindingIdentifier(id) => {
            Vertex::AssignmentToIdentifier(id.span, &id.name)
        }
        BindingPatternKind::ArrayPattern(ap) => {
            Vertex::DestructuringAssignment(DestructuringAssignment::ArrayPattern(ap))
        }
        BindingPatternKind::ObjectPattern(op) => {
            Vertex::DestructuringAssignment(DestructuringAssignment::ObjectPattern(op))
        }
        BindingPatternKind::AssignmentPattern(ap) => {
            Vertex::DestructuringAssignment(DestructuringAssignment::Defaulted(ap))
        }
    }
}

impl<'a> Vertex<'a> {
    fn ts_type_annotation_coercion(&self) -> bool {
        match self {
            Vertex::TSTypeAnnotation(_) => true,
            Vertex::AstNode(ast) => matches!(ast.kind(), AstKind::TSTypeAnnotation(_)),
            _ => false,
        }
    }

    fn ts_type_annotation_type_annotation<'b>(&self) -> VertexIterator<'b, Vertex<'a>>
    where
        'a: 'b,
    {
        Box::new(std::iter::once(Vertex::TSType(match self {
            Vertex::TSTypeAnnotation(tsta) => &tsta.type_annotation,
            Vertex::AstNode(ast) => match ast.kind() {
                AstKind::TSTypeAnnotation(tsta) => &tsta.type_annotation,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        })))
    }

    fn function_coercion(&self) -> bool {
        match self {
            Vertex::Expression(expr) => {
                matches!(expr, Expression::FunctionExpression(_) | Expression::ArrowExpression(_))
            }
            Vertex::AstNode(ast) => {
                matches!(ast.kind(), AstKind::ArrowExpression(_) | AstKind::Function(_))
            }
            _ => false,
        }
    }

    fn function_is_async(&self) -> FieldValue {
        match self {
            Vertex::Expression(expr) => match expr {
                Expression::FunctionExpression(fe) => fe.r#async,
                Expression::ArrowExpression(ae) => ae.r#async,
                _ => unreachable!(),
            },
            Vertex::AstNode(ast) => match ast.kind() {
                AstKind::ArrowExpression(ae) => ae.r#async,
                AstKind::Function(fun) => fun.r#async,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
        .into()
    }

    fn function_signature_arguments<'b>(&self) -> VertexIterator<'b, Vertex<'a>>
    where
        'a: 'b,
    {
        formal_parameters_to_iter(
            match self {
                Vertex::Expression(expr) => match expr {
                    Expression::FunctionExpression(fe) => &fe.0.params,
                    Expression::ArrowExpression(ae) => &ae.0.params,
                    _ => unreachable!(),
                },
                Vertex::AstNode(ast) => match ast.kind() {
                    AstKind::Function(fun) => &fun.params,
                    AstKind::ArrowExpression(ae) => &ae.params,
                    AstKind::TSMethodSignature(tsms) => &tsms.params,
                    _ => unreachable!(),
                },
                Vertex::TSSignature(TSSignature::TSMethodSignature(method_signature)) => {
                    &method_signature.0.params
                }
                _ => unreachable!(),
            }
            .0, // unbox
        )
    }

    fn function_signature_return_type<'b>(&self) -> VertexIterator<'b, Vertex<'a>>
    where
        'a: 'b,
    {
        let return_type = &match self {
            Vertex::Expression(expr) => match expr {
                Expression::FunctionExpression(fe) => &fe.0.return_type,
                Expression::ArrowExpression(ae) => &ae.0.return_type,
                _ => unreachable!(),
            },
            Vertex::AstNode(ast) => match ast.kind() {
                AstKind::Function(fun) => &fun.return_type,
                AstKind::ArrowExpression(ae) => &ae.return_type,
                AstKind::TSMethodSignature(tsms) => &tsms.return_type,
                _ => unreachable!(),
            },
            Vertex::TSSignature(TSSignature::TSMethodSignature(tsms)) => &tsms.0.return_type,
            _ => unreachable!(),
        };

        once_vertex_iter_if_some!(return_type, TSTypeAnnotation)
    }

    fn binary_expression_left(&self) -> &'a Expression<'a> {
        ast_node_or_expression_field!(self, BinaryExpression, left)
    }

    fn binary_expression_right(&self) -> &'a Expression<'a> {
        ast_node_or_expression_field!(self, BinaryExpression, right)
    }

    fn regexp_resolve_coercion(&self) -> bool {
        match &self {
            Vertex::AstNode(node) if matches!(node.kind(), AstKind::RegExpLiteral(_)) => true,
            Vertex::AstNode(node)
                if matches!(node.kind(), AstKind::NewExpression(newexpr)
                if matches!(&newexpr.callee, Expression::Identifier(identifier)
                    if identifier.name.as_str() == "RegExp") &&
                   matches!(newexpr.arguments.first(), Some(arg)
                    if matches!(arg, Argument::Expression(expr) // there must be a constant string for the first argument
                      if matches!(expr, Expression::StringLiteral(_)) || // if a string literal that's fine
                         matches!(expr, Expression::TemplateLiteral(tlit)
                          if !matches!( // it has to be a constant templated string
                              field_value_from_template_lit(tlit),
                              FieldValue::Null)
                          )))) =>
            {
                true
            }
            Vertex::Expression(expr) if matches!(expr, Expression::RegExpLiteral(_)) => true,
            Vertex::TSLiteral(expr, ..) if matches!(expr, TSLiteral::RegExpLiteral(_)) => true,
            _ => false,
        }
    }

    fn regexp_pattern(&self) -> FieldValue {
        match self {
            Vertex::AstNode(node) => match node.kind() {
                AstKind::RegExpLiteral(re) => re.regex.pattern.to_string().into(),
                AstKind::NewExpression(newexpr) => {
                    let Some(arg) = newexpr.arguments.first() else {unreachable!("Got None from the first argument of a NewExpr that should be coercible to a string according to Regexp coercion rules")};
                    let Argument::Expression(expr) = arg else {unreachable!("Got a spread element from the first argument of a NewExpr that should be coercible to a ArgumentExpression according to Regexp coercion rules")};
                    let value = Self::expression_to_constant_string_or_null(expr);
                    debug_assert!(
                        !matches!(value, FieldValue::Null),
                        "String must always be constant or effectively constant according to coercion rules"
                    );
                    value
                }
                _ => unreachable!(),
            },
            Vertex::Expression(Expression::RegExpLiteral(re)) => {
                re.regex.pattern.to_string().into()
            }
            Vertex::TSLiteral(TSLiteral::RegExpLiteral(rlit, ..), ..) => {
                rlit.regex.pattern.to_string().into()
            }
            _ => unreachable!(),
        }
    }

    fn identifier_reference_resolve_coercion(&self) -> bool {
        match &self {
            Vertex::AstNode(ast_node)
                if matches!(ast_node.kind(), AstKind::IdentifierReference(_)) =>
            {
                true
            }
            Vertex::Expression(Expression::Identifier(_)) => true,
            _ => false,
        }
    }

    fn identifier_reference_name(&self) -> &'a str {
        match self {
            Vertex::AstNode(ast_node) => match ast_node.kind() {
                AstKind::IdentifierReference(ir) => &ir.name,
                _ => unreachable!(),
            },
            Vertex::Expression(Expression::Identifier(id)) => &id.0.name,
            _ => unreachable!(),
        }
    }

    fn string_resolve_coercion(&self) -> bool {
        match &self {
            Vertex::AstNode(node) => {
                matches!(node.kind(), AstKind::StringLiteral(_) | AstKind::TemplateLiteral(_))
            }
            Vertex::Expression(expr) => {
                matches!(expr, Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
            }
            Vertex::TSLiteral(tslit, _) => {
                matches!(tslit, TSLiteral::StringLiteral(_) | TSLiteral::TemplateLiteral(_))
            }
            _ => false,
        }
    }

    fn expression_to_constant_string_or_null(expr: &Expression<'a>) -> FieldValue {
        string_matcher!(self, Expression, expr)
    }

    fn string_constant_value(&self) -> FieldValue {
        match &self {
            Vertex::AstNode(node) => string_matcher!(self, AstKind, node.kind()),
            Vertex::Expression(expr) => Self::expression_to_constant_string_or_null(expr),
            Vertex::TSLiteral(tslit, _) => string_matcher!(self, TSLiteral, tslit),
            Vertex::EffectivelyLiteralString(_, atom) => atom.as_str().to_string().into(),
            _ => unreachable!(),
        }
    }

    fn get_span_from_ast_node<'b>(&self) -> VertexIterator<'b, Vertex<'a>>
    where
        'a: 'b,
    {
        Box::new(std::iter::once(Vertex::Span(match self {
            Vertex::AstNode(ast) => ast.kind().span(),
            Vertex::Expression(expr) => expr.span(),
            Vertex::Argument(arg, _) => arg.span(),
            Vertex::AssignmentToIdentifier(span, _)
            | Vertex::TSLiteral(_, span)
            | Vertex::TSTypeName(_, span)
            | Vertex::EffectivelyLiteralString(span, _)
            | Vertex::PropertyAccess(span, _) => *span,
            Vertex::File | Vertex::Span(_) | Vertex::Operator(_) | Vertex::PossiblyNumber(_) => {
                unreachable!(
                    "tried to get span out of Vertex::Span | Vertex::Operator | Vertex::PossiblyNumber | Vertex::File"
                )
            }
            Vertex::AssignmentOnObject(assignment) => match &assignment {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => {
                    unreachable!("handled by AssignmentToIdentifier vertex")
                }
                SimpleAssignmentTarget::MemberAssignmentTarget(member) => member.0.span(),
                _ => assignment.span(),
            },
            Vertex::Parameter(param, _) => param.span(),
            Vertex::AssignmentOperator(aso) => aso.span,
            Vertex::PropertyKey(pk) => pk.span(),
            Vertex::TSType(tt) => tt.span(),
            Vertex::TSSignature(tss) => tss.span(),
            Vertex::TSTypeAnnotation(tst) => tst.span,
            Vertex::ImportSpecifier(sl) => sl.span(),
            Vertex::JsxElementName(sl) => sl.span(),
            Vertex::JsxOpeningElement(joe) => joe.span,
            Vertex::JsxClosingElement(joe) => joe.span,
            Vertex::DestructuringAssignment(da) => da.span(),
            Vertex::ObjectProperty(op) => op.span(),
            Vertex::VariableDeclaration(vd) => vd.span,
        })))
    }
}

impl<'b, 'a: 'b> Adapter<'b> for &'b LintAdapter<'a> {
    type Vertex = Vertex<'a>;

    fn resolve_starting_vertices(
        &self,
        edge_name: &Arc<str>,
        _parameters: &EdgeParameters,
        _resolve_info: &ResolveInfo,
    ) -> VertexIterator<'b, Self::Vertex> {
        match edge_name.as_ref() {
            "File" => Box::new(std::iter::once(Vertex::File)),
            _ => unimplemented!("unexpected starting edge: {edge_name}"),
        }
    }

    fn resolve_property(
        &self,
        contexts: ContextIterator<'b, Self::Vertex>,
        type_name: &Arc<str>,
        property_name: &Arc<str>,
        _resolve_info: &ResolveInfo,
    ) -> ContextOutcomeIterator<'b, Self::Vertex, FieldValue> {
        let res = match (type_name.as_ref(), property_name.as_ref()) {
            ("Identifier", "name") => resolve_property_with(contexts, |v| {
                let Expression::Identifier(ident) = v.as_expression().unwrap() else { unreachable!("expected identifier") };
                ident.name.to_string().into()
            }),
            ("AssignmentToIdentifier", "name") => resolve_property_with(contexts, |v| {
                v.as_assignment_to_identifier().unwrap().1.to_string().into()
            }),
            ("Function", "is_async") => resolve_property_with(contexts, Vertex::function_is_async),
            ("Span", "start") => {
                resolve_property_with(contexts, |v| v.as_span().unwrap().start.into())
            }
            ("Span", "end") => resolve_property_with(contexts, |v| v.as_span().unwrap().end.into()),
            ("Operator", "is_equality") => {
                resolve_property_with(contexts, |v| v.as_operator().unwrap().is_equality().into())
            }
            ("Operator", "is_bitwise") => {
                resolve_property_with(contexts, |v| v.as_operator().unwrap().is_bitwise().into())
            }
            ("AssignmentOperator", "str") => resolve_property_with(contexts, |v| {
                v.as_assignment_operator().unwrap().operator.as_str().into()
            }),
            ("AssignmentOperator", "is_bitwise") => resolve_property_with(contexts, |v| {
                v.as_assignment_operator().unwrap().operator.is_bitwise().into()
            }),
            ("Operator", "str") => {
                resolve_property_with(contexts, |v| v.as_operator().unwrap().as_str().into())
            }
            ("ExpressionArgument", "index") => {
                resolve_property_with(contexts, |v| (*v.as_argument().unwrap().1).into())
            }
            ("Parameter", "index") => {
                resolve_property_with(contexts, |v| (*v.as_parameter().unwrap().1).into())
            }
            ("IdentifierParameter", "name") => resolve_property_with(contexts, |v| {
                let BindingPatternKind::BindingIdentifier(ident) = v.as_parameter().unwrap().0 else { unreachable!() };
                ident.name.to_string().into()
            }),
            ("PossiblyNumber", "value") => resolve_property_with(contexts, |v| {
                let num = v.as_possibly_number().unwrap();
                if num.is_finite() { FieldValue::Float64(*num) } else { FieldValue::Null }
            }),
            ("IdentifierPropertyKey", "identifier") => resolve_property_with(contexts, |v| {
                let PropertyKey::Identifier(ident) = v.as_property_key().unwrap() else { unreachable!() };

                ident.name.to_string().into()
            }),
            ("PropertyAccess", "name") => resolve_property_with(contexts, |v| {
                (*v.as_property_access().unwrap().1).to_string().into()
            }),
            ("StringASTNode" | "StringExpression" | "ConstantString", "constant_value") => {
                resolve_property_with(contexts, Vertex::string_constant_value)
            }
            ("IdentifierReference", "name") => {
                resolve_property_with(contexts, |v| Vertex::identifier_reference_name(v).into())
            }
            ("RegExp" | "RegExpLiteral", "pattern") => {
                resolve_property_with(contexts, crate::lint_adapter::Vertex::regexp_pattern)
            }
            ("TSInterfaceDeclaration", "name") => resolve_property_with(contexts, |v| {
                let AstKind::TSInterfaceDeclaration(tsid) = v.as_ast_node().unwrap().kind() else {unreachable!()};
                tsid.id.name.to_string().into()
            }),
            ("ImportDeclaration", "is_value_import") => resolve_property_with(contexts, |v| {
                let AstKind::ModuleDeclaration(md) = v.as_ast_node().unwrap().kind() else { unreachable!("expected module declaration") };
                let ModuleDeclaration::ImportDeclaration(id) = md else { unreachable!("expected import declaration") };
                matches!(id.import_kind, ImportOrExportKind::Value).into()
            }),
            ("ImportDeclaration", "is_type_import") => resolve_property_with(contexts, |v| {
                let AstKind::ModuleDeclaration(md) = v.as_ast_node().unwrap().kind() else { unreachable!("expected module declaration") };
                let ModuleDeclaration::ImportDeclaration(id) = md else { unreachable!("expected import declaration") };
                matches!(id.import_kind, ImportOrExportKind::Type).into()
            }),
            ("TSPropertySignature", "is_optional") => resolve_property_with(contexts, |v| {
                let property_signature = get_ast_or_x!(v, TSSignature, TSPropertySignature);
                property_signature.optional.into()
            }),
            ("TSIdentifierReference", "name") => resolve_property_with(contexts, |v| {
                let tstn = match *v {
                    Vertex::TSTypeName(tstn, ..) => tstn,
                    Vertex::AstNode(_) | Vertex::TSType(_) => {
                        &get_ast_or_x!(v, TSType, TSTypeReference).type_name
                    }
                    _ => unreachable!(),
                };
                let TSTypeName::IdentifierName(ident) = tstn else { unreachable!() };
                ident.0.name.to_string().into()
            }),
            ("FunctionSignature" | "Function" | "TSMethodSignature", "name") => {
                resolve_property_with(contexts, |v| match v {
                    Vertex::Expression(expr) => match expr {
                        Expression::FunctionExpression(fe) => {
                            fe.0.id
                                .as_ref()
                                .map_or_else(|| FieldValue::Null, |id| id.name.to_string().into())
                        }
                        Expression::ArrowExpression(_) => FieldValue::Null,
                        _ => unreachable!(),
                    },
                    Vertex::AstNode(ast) => match ast.kind() {
                        AstKind::Function(fun) => fun
                            .id
                            .as_ref()
                            .map_or_else(|| FieldValue::Null, |id| id.name.to_string().into()),
                        AstKind::ArrowExpression(_) => {
                            // TODO: Look at parent node for assignment for name
                            FieldValue::Null
                            // match self.semantic.nodes().parent_node(ast.id()) {
                            //     Some(parent) => match parent.kind() {
                            //         AstKind::BindingIdentifier(ident) => ident.name.to_string().into(),
                            //         AstKind::IdentifierReference(ident) => {
                            //             ident.name.to_string().into()
                            //         }
                            //         _ => FieldValue::Null,
                            //     },
                            //     None => FieldValue::Null,
                            // }
                        }
                        AstKind::TSMethodSignature(tsms) => tsms
                            .key
                            .static_name()
                            .map_or_else(|| FieldValue::Null, |id| id.to_string().into()),
                        _ => unreachable!(),
                    },
                    Vertex::TSSignature(TSSignature::TSMethodSignature(method_signature)) => {
                        method_signature
                            .key
                            .static_name()
                            .map_or_else(|| FieldValue::Null, |id| id.to_string().into())
                    }
                    _ => unreachable!(),
                })
            }
            _ => unimplemented!("unexpected property: {type_name} {property_name}"),
        };

        res
    }

    fn resolve_neighbors(
        &self,
        contexts: ContextIterator<'b, Self::Vertex>,
        type_name: &Arc<str>,
        edge_name: &Arc<str>,
        _parameters: &EdgeParameters,
        _resolve_info: &ResolveEdgeInfo,
    ) -> ContextOutcomeIterator<'b, Self::Vertex, VertexIterator<'b, Self::Vertex>> {
        let res = match (type_name.as_ref(), edge_name.as_ref()) {
            ("File", "ast_node") => resolve_neighbors_with(contexts, |_| {
                Box::new(self.semantic.nodes().iter().map(|x| Vertex::AstNode(*x)))
            }),
            ("AssignmentOnObject", "final_assignment") => resolve_neighbors_with(contexts, |v| {
                let first = v.as_assignment_on_object().unwrap();
                let object = match &first {
                    SimpleAssignmentTarget::AssignmentTargetIdentifier(_) => {
                        unreachable!(
                            "This should be handled by AssignmentToIdentifier vertex, and not here"
                        )
                    }
                    SimpleAssignmentTarget::MemberAssignmentTarget(target) => target.0.object(),
                    _ => first.get_expression().unwrap(),
                };

                Box::new(std::iter::once(Vertex::Expression(solve_member_expr_for_lowest(object))))
            }),
            ("FieldAccessOnObject", "final_assignment") => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(Vertex::Expression(solve_member_expr_for_lowest(
                    get_ast_or_x!(v, Expression, MemberExpression).object(),
                ))))
            }),
            ("FieldAccessOnObject", "property_name") => resolve_neighbors_with(contexts, |v| {
                let (a, b) =
                    get_ast_or_x!(v, Expression, MemberExpression).static_property_info().unwrap();
                Box::new(std::iter::once(Vertex::PropertyAccess(a, b)))
            }),
            ("CallExpression", "arguments") => resolve_neighbors_with(contexts, |v| {
                Box::new(
                    get_ast_or_x!(v, Expression, CallExpression)
                        .arguments
                        .iter()
                        .enumerate()
                        .map(|(ix, arg)| Vertex::Argument(arg, ix.try_into().unwrap())),
                )
            }),
            ("NewExpression", "callee") => resolve_neighbors_with(contexts, |v| {
                let v = v.as_ast_node().unwrap();
                let AstKind::NewExpression(expr) = v.kind() else { unreachable!() };

                Box::new(std::iter::once(Vertex::Expression(&expr.callee)))
            }),
            ("ClassPropertyDefinition", "key") => resolve_neighbors_with(contexts, |v| {
                let v = v.as_ast_node().unwrap();
                let AstKind::PropertyDefinition(expr) = v.kind() else { unreachable!() };

                Box::new(std::iter::once(Vertex::PropertyKey(&expr.key)))
            }),
            ("TSTypeAnnotation", "ts_type") => {
                resolve_neighbors_with(contexts, Vertex::ts_type_annotation_type_annotation)
            }
            ("TSPropertySignature", "type_annotation") => resolve_neighbors_with(contexts, |v| {
                let v = v.as_ast_node().unwrap();
                let AstKind::TSPropertySignature(expr) = v.kind() else { unreachable!() };

                once_vertex_iter_if_some!(expr.type_annotation, TSTypeAnnotation)
            }),
            ("NewExpression", "arguments") => resolve_neighbors_with(contexts, |v| {
                let v = v.as_ast_node().unwrap();
                let AstKind::NewExpression(expr) = v.kind() else { unreachable!() };

                Box::new(
                    expr.arguments
                        .iter()
                        .enumerate()
                        .map(|(ix, arg)| Vertex::Argument(arg, ix.try_into().unwrap())),
                )
            }),
            ("IfStatementOrTernary" | "IfStatement" | "ConditionalExpression", "test") => {
                resolve_neighbors_with(contexts, |v| match v {
                    Vertex::AstNode(node) => match node.kind() {
                        AstKind::IfStatement(expr) => {
                            Box::new(std::iter::once(Vertex::Expression(&expr.test)))
                        }
                        AstKind::ConditionalExpression(condexpr) => {
                            Box::new(std::iter::once(Vertex::Expression(&condexpr.test)))
                        }
                        _ => unreachable!(),
                    },
                    Vertex::Expression(expr) => {
                        let Expression::ConditionalExpression(condexpr) = expr else { unreachable!() };
                        Box::new(std::iter::once(Vertex::Expression(&condexpr.test)))
                    }
                    _ => unreachable!(),
                })
            }
            ("FieldAccessOnObject", "called_on") => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(Vertex::Expression(
                    get_ast_or_x!(v, Expression, MemberExpression).object(),
                )))
            }),
            ("ObjectExpression", "properties") => resolve_neighbors_with(contexts, |v| {
                Box::new(
                    get_ast_or_x!(v, Expression, ObjectExpression)
                        .properties
                        .iter()
                        .map(Vertex::ObjectProperty),
                )
            }),
            ("CallExpression", "callee") => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(Vertex::Expression(
                    &get_ast_or_x!(v, Expression, CallExpression).callee,
                )))
            }),
            (
                "ExpressionArgument"
                | "Expression"
                | "BinaryExpression"
                | "FieldAccessOnObject"
                | "CallExpression"
                | "AwaitExpression",
                "inner_expr",
            ) => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(Vertex::Expression(match v {
                    Vertex::AstNode(astnode) => match astnode.kind() {
                        AstKind::AssignmentExpression(ase) => ase.right.get_inner_expression(),
                        AstKind::CallExpression(cexpr) => cexpr.callee.get_inner_expression(),
                        AstKind::ParenthesizedExpression(pexpr) => {
                            pexpr.expression.get_inner_expression()
                        }
                        AstKind::UnaryExpression(uexpr) => uexpr.argument.get_inner_expression(),
                        AstKind::AwaitExpression(aexpr) => aexpr.argument.get_inner_expression(),
                        _ => unreachable!(),
                    },
                    Vertex::Expression(expr) => expr.get_inner_expression(),
                    Vertex::Argument(arg, _) => {
                        let Argument::Expression(expr_arg) = arg else { unreachable!("expected expression argument") };
                        expr_arg.get_inner_expression()
                    }
                    _ => unreachable!(),
                })))
            }),
            ("AssignmentExpression", "left") => resolve_neighbors_with(contexts, |v| {
                let assignment_expression = get_ast_or_x!(v, Expression, AssignmentExpression);

                Box::new(std::iter::once(assignment_target_to_vertex(&assignment_expression.left)))
            }),
            ("AssignmentExpression", "right") => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(Vertex::Expression(ast_node_or_expression_field!(
                    v,
                    AssignmentExpression,
                    right
                ))))
            }),
            ("AssignmentExpression", "operator") => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(Vertex::AssignmentOperator(get_ast_or_x!(
                    v,
                    Expression,
                    AssignmentExpression
                ))))
            }),
            ("VariableDeclarations", "declarations") => resolve_neighbors_with(contexts, |v| {
                let AstKind::VariableDeclaration(decl) = v.as_ast_node().unwrap().kind() else { unreachable!() };
                Box::new(decl.declarations.iter().map(Vertex::VariableDeclaration))
            }),
            ("VariableDeclaration", "left_type") => resolve_neighbors_with(contexts, |v| {
                let decl = v.as_variable_declaration().unwrap();
                once_vertex_iter_if_some!(decl.id.type_annotation, TSTypeAnnotation)
            }),
            ("VariableDeclaration", "left") => resolve_neighbors_with(contexts, |v| {
                let decl = v.as_variable_declaration().unwrap();
                Box::new(std::iter::once(match &decl.id.kind {
                    BindingPatternKind::BindingIdentifier(id) => {
                        Vertex::AssignmentToIdentifier(id.span, &id.name)
                    }
                    BindingPatternKind::ArrayPattern(ap) => {
                        Vertex::DestructuringAssignment(DestructuringAssignment::ArrayPattern(ap))
                    }
                    BindingPatternKind::ObjectPattern(op) => {
                        Vertex::DestructuringAssignment(DestructuringAssignment::ObjectPattern(op))
                    }
                    BindingPatternKind::AssignmentPattern(ap) => {
                        Vertex::DestructuringAssignment(DestructuringAssignment::Defaulted(ap))
                    }
                }))
            }),
            (_, "span") => resolve_neighbors_with(contexts, Vertex::get_span_from_ast_node),
            ("BinaryExpression", "left") => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(Vertex::Expression(v.binary_expression_left())))
            }),
            ("BinaryExpression", "right") => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(v.binary_expression_right()).map(Vertex::Expression))
            }),
            ("BinaryExpression", "both_sides") => resolve_neighbors_with(contexts, |v| {
                Box::new(
                    vec![v.binary_expression_left(), v.binary_expression_right()]
                        .into_iter()
                        .map(Vertex::Expression),
                )
            }),
            ("BinaryExpression", "operator") => resolve_neighbors_with(contexts, |v| {
                Box::new(std::iter::once(Vertex::Operator(ast_node_or_expression_field!(
                    v,
                    BinaryExpression,
                    operator
                ))))
            }),
            ("FunctionSignature" | "Function" | "TSMethodSignature", "parameters") => {
                resolve_neighbors_with(contexts, Vertex::function_signature_arguments)
            }
            ("FunctionSignature" | "Function" | "TSMethodSignature", "return_type") => {
                resolve_neighbors_with(contexts, Vertex::function_signature_return_type)
            }
            ("TSMethodSignature", "key") => resolve_neighbors_with(contexts, |v| {
                let method_signature = get_ast_or_x!(v, TSSignature, TSMethodSignature);
                Box::new(std::iter::once(Vertex::PropertyKey(&method_signature.key)))
            }),
            ("TSLiteralType", "expr") => resolve_neighbors_with(contexts, |v| {
                let literal_type = get_ast_or_x!(v, TSType, TSLiteralType);
                Box::new(std::iter::once(Vertex::TSLiteral(
                    &literal_type.literal,
                    literal_type.span,
                )))
            }),
            ("TSUnionType", "subtypes") => resolve_neighbors_with(contexts, |v| {
                let union_type = get_ast_or_x!(v, TSType, TSUnionType);
                Box::new(union_type.types.iter().map(Vertex::TSType))
            }),
            ("NumberLiteral", "value") => resolve_neighbors_with(contexts, |v| {
                let i = get_ast_or_x_or_y!(v, Expression, TSLiteral, NumberLiteral);
                Box::new(std::iter::once(Vertex::PossiblyNumber(i.value)))
            }),
            ("TSInterfaceDeclaration", "body") => resolve_neighbors_with(contexts, |v| {
                let AstKind::TSInterfaceDeclaration(tsid) = v.as_ast_node().unwrap().kind() else {unreachable!()};
                Box::new(tsid.body.body.iter().map(Vertex::TSSignature))
            }),
            ("JSXOpeningElement", "name") => resolve_neighbors_with(contexts, |v| {
                let AstKind::JSXOpeningElement(joe) = v.as_ast_node().unwrap().kind() else {unreachable!()};
                Box::new(std::iter::once(Vertex::JsxElementName(&joe.name)))
            }),
            ("JSXSimpleName", "name") => resolve_neighbors_with(contexts, |v| {
                let JSXElementName::Identifier(ident) = v.as_jsx_element_name().unwrap() else {unreachable!()};
                Box::new(std::iter::once(Vertex::EffectivelyLiteralString(ident.span, &ident.name)))
            }),
            ("JSXElement", "opening_element") => resolve_neighbors_with(contexts, |v| {
                let Expression::JSXElement(el) = v.as_expression().unwrap() else {unreachable!()};
                Box::new(std::iter::once(Vertex::JsxOpeningElement(el.opening_element.0)))
            }),
            ("JSXElement", "closing_element") => resolve_neighbors_with(contexts, |v| {
                let Expression::JSXElement(el) = v.as_expression().unwrap() else {unreachable!()};
                once_vertex_iter_if_some!(el.closing_element, JsxClosingElement)
            }),
            ("TSTypeReference", "name") => resolve_neighbors_with(contexts, |v| {
                let tstr = get_ast_or_x!(v, TSType, TSTypeReference);
                Box::new(std::iter::once(Vertex::TSTypeName(&tstr.type_name, tstr.span)))
            }),
            ("AwaitExpression", "expression") => resolve_neighbors_with(contexts, |v| {
                let await_expr = get_ast_or_x!(v, Expression, AwaitExpression);
                Box::new(std::iter::once(Vertex::Expression(&await_expr.argument)))
            }),
            ("ImportDeclaration", "source") => resolve_neighbors_with(contexts, |v| {
                let AstKind::ModuleDeclaration(md) = v.as_ast_node().unwrap().kind() else { unreachable!("expected module declaration") };
                let ModuleDeclaration::ImportDeclaration(id) = md else { unreachable!("expected import declaration") };
                let StringLiteral { span, value } = &id.source;
                Box::new(std::iter::once(Vertex::EffectivelyLiteralString(*span, value)))
            }),
            ("ImportDeclaration", "specifiers") => resolve_neighbors_with(contexts, |v| {
                let AstKind::ModuleDeclaration(md) = v.as_ast_node().unwrap().kind() else { unreachable!("expected module declaration") };
                let ModuleDeclaration::ImportDeclaration(id) = md else { unreachable!("expected import declaration") };
                Box::new(id.specifiers.iter().map(Vertex::ImportSpecifier))
            }),
            ("ObjectDestructingAssignment" | "ArrayDestructuringAssignment", "rest") => {
                resolve_neighbors_with(contexts, |v| {
                    let rest = match v.as_destructuring_assignment().unwrap() {
                        DestructuringAssignment::AssignmentExpression(ae) => {
                            let temp = match ae {
                                AssignmentTargetPattern::ArrayAssignmentTarget(aat) => &aat.rest,
                                AssignmentTargetPattern::ObjectAssignmentTarget(oat) => &oat.rest,
                            };

                            temp.as_ref().map(|assignment_target| {
                                assignment_target_to_vertex(assignment_target)
                            })
                        }
                        DestructuringAssignment::ArrayPattern(ap) => ap
                            .rest
                            .as_ref()
                            .map(|rest| binding_pattern_kind_to_vertex(&rest.0.argument.kind)),
                        DestructuringAssignment::ObjectPattern(op) => op
                            .rest
                            .as_ref()
                            .map(|rest| binding_pattern_kind_to_vertex(&rest.0.argument.kind)),
                        DestructuringAssignment::Defaulted(_) => None,
                    };

                    #[allow(clippy::option_if_let_else)]
                    if let Some(rest) = rest {
                        Box::new(std::iter::once(rest))
                    } else {
                        Box::new(std::iter::empty())
                    }
                })
            }
            _ => unimplemented!("unexpected neighbor: {type_name} {edge_name}"),
        };

        res
    }

    fn resolve_coercion(
        &self,
        contexts: ContextIterator<'b, Self::Vertex>,
        _type_name: &Arc<str>,
        coerce_to_type: &Arc<str>,
        _resolve_info: &ResolveInfo,
    ) -> ContextOutcomeIterator<'b, Self::Vertex, bool> {
        match coerce_to_type.as_ref() {
            "NewExpression" => coerce_ast!(contexts, NewExpression),
            "ExpressionArgument" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::Argument(arg, _) if matches!(arg, Argument::Expression(_))),
            ),
            "Function" => resolve_coercion_with(contexts, Vertex::function_coercion),
            "AssignmentExpression" => coerce_ast_or_x!(contexts, Expression, AssignmentExpression),
            "ConditionalExpression" => {
                coerce_ast_or_x!(contexts, Expression, ConditionalExpression)
            }
            "IfStatement" => coerce_ast!(contexts, IfStatement),
            "ForInStatement" => coerce_ast!(contexts, ForInStatement),
            "AssignmentToIdentifier" => resolve_coercion_with(contexts, |v| {
                matches!(v, Vertex::AssignmentToIdentifier(_, _))
            }),
            "AssignmentOnObject" => {
                resolve_coercion_with(contexts, |v| matches!(v, Vertex::AssignmentOnObject(_)))
            }
            "BinaryExpression" => coerce_ast_or_x!(contexts, Expression, BinaryExpression),
            "IfStatementOrTernary" => resolve_coercion_with(contexts, |v| match v {
                Vertex::AstNode(node) => matches!(
                    node.kind(),
                    AstKind::IfStatement(_) | AstKind::ConditionalExpression(_)
                ),
                Vertex::Expression(expr) => {
                    matches!(expr, Expression::ConditionalExpression(_))
                }
                _ => false,
            }),
            "FieldAccessOnObject" => coerce_ast_or_x!(contexts, Expression, MemberExpression),
            "ObjectExpression" => coerce_ast_or_x!(contexts, Expression, ObjectExpression),
            "NumberLiteral" => {
                coerce_ast_or_x_or_y!(contexts, Expression, TSLiteral, NumberLiteral)
            }
            "CallExpression" => coerce_ast_or_x!(contexts, Expression, CallExpression),
            "UnaryExpression" => coerce_ast_or_x!(contexts, Expression, UnaryExpression),
            "RegExp" => resolve_coercion_with(contexts, Vertex::regexp_resolve_coercion),
            "RegExpLiteral" => coerce_ast_or_x!(contexts, Expression, RegExpLiteral),
            "ClassPropertyDefinition" => coerce_ast!(contexts, PropertyDefinition),
            "ExportDefault" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::AstNode(node) if matches!(node.kind(), AstKind::ModuleDeclaration(md) if matches!(md, ModuleDeclaration::ExportDefaultDeclaration(_)))),
            ),
            "IdentifierPropertyKey" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::PropertyKey(pk) if matches!(pk, PropertyKey::Identifier(_))),
            ),
            "StringASTNode" | "StringExpression" | "ConstantString" => {
                resolve_coercion_with(contexts, Vertex::string_resolve_coercion)
            }
            "Identifier" => {
                resolve_coercion_with(contexts, Vertex::identifier_reference_resolve_coercion)
            }
            "TSTypeAnnotation" => {
                resolve_coercion_with(contexts, Vertex::ts_type_annotation_coercion)
            }
            "TSAny" => coerce_ast_or_x!(contexts, TSType, TSAnyKeyword),
            "TSLiteralType" => coerce_ast_or_x!(contexts, TSType, TSLiteralType),
            "TSUnionType" => coerce_ast_or_x!(contexts, TSType, TSUnionType),
            "TSInterfaceDeclaration" => coerce_ast!(contexts, TSInterfaceDeclaration),
            "TSPropertySignature" => coerce_ast_or_x!(contexts, TSSignature, TSPropertySignature),
            "TSMethodSignature" => coerce_ast_or_x!(contexts, TSSignature, TSMethodSignature),
            "IdentifierParameter" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::Parameter(param, _) if matches!(param, BindingPatternKind::BindingIdentifier(_))),
            ),
            "TSTypeReference" => coerce_ast_or_x!(contexts, TSType, TSTypeReference),
            "TSIdentifierReference" => resolve_coercion_with(contexts, |v| match *v {
                Vertex::TSType(tst) if matches!(tst, TSType::TSTypeReference(tsr) if matches!(tsr.type_name, TSTypeName::IdentifierName(..))) => {
                    true
                }
                Vertex::TSTypeName(tstn, ..) if matches!(&tstn, TSTypeName::IdentifierName(..)) => {
                    true
                }
                Vertex::AstNode(node) if matches!(node.kind(), AstKind::TSTypeReference(tsr) if matches!(tsr.type_name, TSTypeName::IdentifierName(..))) => {
                    true
                }
                _ => false,
            }),
            "FunctionSignature" => resolve_coercion_with(contexts, |v| {
                Vertex::function_coercion(v)
                    || coerce_ast_or_x_match!(v, TSSignature, TSMethodSignature)
            }),
            "ImportDeclaration" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::AstNode(node) if matches!(node.kind(), AstKind::ModuleDeclaration(md) if matches!(md, ModuleDeclaration::ImportDeclaration(_)))),
            ),
            "ImportStarSpecifier" => resolve_coercion_with(contexts, |v| {
                let import_specifier = v.as_import_specifier();
                let Some(import_specifier) = import_specifier else {return false};
                matches!(import_specifier, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
            }),
            "ImportDefaultSpecifier" => resolve_coercion_with(contexts, |v| {
                let import_specifier = v.as_import_specifier();
                let Some(import_specifier) = import_specifier else {return false};
                matches!(import_specifier, ImportDeclarationSpecifier::ImportDefaultSpecifier(_))
            }),
            "ImportSpecificSpecifier" => resolve_coercion_with(contexts, |v| {
                let import_specifier = v.as_import_specifier();
                let Some(import_specifier) = import_specifier else {return false};
                matches!(import_specifier, ImportDeclarationSpecifier::ImportSpecifier(_))
            }),
            "JSXOpeningElement" => coerce_ast!(contexts, JSXOpeningElement),
            "JSXSimpleName" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::JsxElementName(op) if matches!(op, JSXElementName::Identifier(_))),
            ),
            "JSXElement" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::Expression(op) if matches!(op, Expression::JSXElement(_))),
            ),
            "ArrayDestructuringAssignment" => resolve_coercion_with(contexts, |v| match v {
                Vertex::DestructuringAssignment(da) => matches!(
                    da,
                    DestructuringAssignment::AssignmentExpression(
                        AssignmentTargetPattern::ArrayAssignmentTarget(_),
                    ) | DestructuringAssignment::ArrayPattern(_)
                ),
                _ => false,
            }),
            "ObjectDestructingAssignment" => {
                resolve_coercion_with(contexts, |v| matches!(v, Vertex::DestructuringAssignment(_)))
            }
            "SingleObjectProperty" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::ObjectProperty(op) if matches!(op, ObjectPropertyKind::ObjectProperty(_))),
            ),
            "SpreadObjectProperty" => resolve_coercion_with(
                contexts,
                |v| matches!(v, Vertex::ObjectProperty(op) if matches!(op, ObjectPropertyKind::SpreadProperty(_))),
            ),
            "VariableDeclarations" => coerce_ast!(contexts, VariableDeclaration),
            "VariableDeclaration" => {
                resolve_coercion_with(contexts, |v| matches!(v, Vertex::VariableDeclaration(_)))
            }
            "AwaitExpression" => coerce_ast_or_x!(contexts, Expression, AwaitExpression),
            _ => unimplemented!("unexpected coercion to: {coerce_to_type}"),
        }
    }
}
