//! Lowering of AST `TSType` nodes into the owned IR.
//!
//! Used in two places with different name-resolution strategies:
//! - surface extraction (pass A): names resolve to *pending* refs, fixed up by
//!   the linker once all files are known;
//! - file checking (pass B): names resolve against the frozen environment plus
//!   the file's local type declarations.

use oxc_ast::ast::{
    FormalParameters, PropertyKey, TSInterfaceDeclaration, TSLiteral, TSSignature, TSTupleElement,
    TSType, TSTypeAnnotation, TSTypeName,
};

use crate::ir::{FunctionShape, Member, ObjectShape, Param, RefTarget, Type, TypeId, TypeTable};

/// Result of resolving a type name during lowering.
pub enum NameTarget {
    /// Becomes a [`Type::Ref`] with this target.
    Ref(RefTarget),
    /// An already-lowered local type (pass B inline aliases). Type arguments
    /// on an inline target lower to [`Type::Unsupported`].
    Inline(TypeId),
    /// An enum member type (`Status.Active` in type position).
    EnumMember {
        /// The enum symbol.
        symbol: crate::ir::SymbolId,
        /// Member index.
        index: u32,
    },
}

/// Name resolution strategy, differing between surface extraction and checking.
pub trait ResolveName {
    /// Resolve a plain identifier in type position.
    fn resolve(&self, name: &str) -> NameTarget;
    /// Resolve a single-level qualified name `left.member` in type position.
    /// `span` is the member name's span, for diagnostics (TS2694).
    fn resolve_qualified(&self, left: &str, member: &str, span: oxc_span::Span) -> NameTarget;
}

/// A growable overlay of types on top of (conceptually) the global table.
///
/// Ids below `base` refer to intrinsics (pass A) or to the frozen global table
/// (pass B); ids at or above `base` index into `types`.
#[derive(Debug)]
pub struct TypeSink {
    base: u32,
    types: Vec<Type>,
}

impl TypeSink {
    /// Create a sink whose first local entry will have id `base`.
    pub fn new(base: u32) -> Self {
        Self { base, types: Vec::new() }
    }

    /// The id of the first local entry.
    pub fn base(&self) -> u32 {
        self.base
    }

    /// Append a type, returning its id.
    pub fn push(&mut self, ty: Type) -> TypeId {
        let id = TypeId(self.base + u32::try_from(self.types.len()).expect("type sink overflow"));
        self.types.push(ty);
        id
    }

    /// Look up a *local* type (id must be `>= base`).
    pub fn get_local(&self, id: TypeId) -> &Type {
        &self.types[(id.0 - self.base) as usize]
    }

    /// Consume the sink, returning the local entries.
    pub fn into_types(self) -> Vec<Type> {
        self.types
    }
}

/// Lowers `TSType` nodes into a [`TypeSink`] using a [`ResolveName`] strategy.
pub struct Lowerer<'a, R> {
    /// Destination for created types.
    pub sink: &'a mut TypeSink,
    /// Name resolution strategy.
    pub resolver: &'a R,
    /// In-scope type parameter names of the enclosing generic declaration;
    /// matching identifiers lower to [`Type::TypeParam`] markers.
    pub type_params: Vec<Box<str>>,
}

impl<'a, R: ResolveName> Lowerer<'a, R> {
    /// Create a lowerer writing into `sink`.
    pub fn new(sink: &'a mut TypeSink, resolver: &'a R) -> Self {
        Self { sink, resolver, type_params: Vec::new() }
    }

    /// Create a lowerer with type parameters in scope.
    pub fn with_params(
        sink: &'a mut TypeSink,
        resolver: &'a R,
        type_params: Vec<Box<str>>,
    ) -> Self {
        Self { sink, resolver, type_params }
    }

    /// Lower an optional annotation, defaulting to `any`.
    pub fn lower_annotation(&mut self, annotation: Option<&TSTypeAnnotation<'_>>) -> TypeId {
        annotation.map_or(TypeTable::ANY, |a| self.lower_type(&a.type_annotation))
    }

    /// Lower a `TSType` to a type id.
    pub fn lower_type(&mut self, ty: &TSType<'_>) -> TypeId {
        match ty {
            TSType::TSAnyKeyword(_) => TypeTable::ANY,
            TSType::TSUnknownKeyword(_) => TypeTable::UNKNOWN,
            TSType::TSNeverKeyword(_) => TypeTable::NEVER,
            TSType::TSVoidKeyword(_) => TypeTable::VOID,
            TSType::TSUndefinedKeyword(_) => TypeTable::UNDEFINED,
            TSType::TSNullKeyword(_) => TypeTable::NULL,
            TSType::TSStringKeyword(_) => TypeTable::STRING,
            TSType::TSNumberKeyword(_) => TypeTable::NUMBER,
            TSType::TSBooleanKeyword(_) => TypeTable::BOOLEAN,
            TSType::TSBigIntKeyword(_) => TypeTable::BIGINT,
            TSType::TSSymbolKeyword(_) => TypeTable::SYMBOL,
            TSType::TSObjectKeyword(_) => TypeTable::OBJECT_KEYWORD,
            TSType::TSLiteralType(lit) => self.lower_literal(&lit.literal),
            TSType::TSUnionType(union) => {
                let members: Vec<TypeId> = union.types.iter().map(|t| self.lower_type(t)).collect();
                self.sink.push(Type::Union(members.into_boxed_slice()))
            }
            TSType::TSIntersectionType(inter) => {
                let members: Vec<TypeId> = inter.types.iter().map(|t| self.lower_type(t)).collect();
                self.sink.push(Type::Intersection(members.into_boxed_slice()))
            }
            TSType::TSArrayType(arr) => {
                let elem = self.lower_type(&arr.element_type);
                self.sink.push(Type::Array(elem))
            }
            TSType::TSTupleType(tuple) => self.lower_tuple(&tuple.element_types),
            TSType::TSTypeReference(reference) => {
                self.lower_reference(&reference.type_name, reference.type_arguments.as_deref())
            }
            TSType::TSTypeLiteral(lit) => self.lower_members(&lit.members, false),
            TSType::TSFunctionType(func) => {
                let shape = self.lower_function_shape(&func.params, Some(&func.return_type));
                self.sink.push(Type::Function(Box::new(shape)))
            }
            TSType::TSParenthesizedType(paren) => self.lower_type(&paren.type_annotation),
            TSType::TSTypeOperatorType(op) => {
                use oxc_ast::ast::TSTypeOperatorOperator;
                match op.operator {
                    TSTypeOperatorOperator::Readonly => {
                        let inner = self.lower_type(&op.type_annotation);
                        self.sink.push(Type::Readonly(inner))
                    }
                    TSTypeOperatorOperator::Keyof | TSTypeOperatorOperator::Unique => {
                        TypeTable::UNSUPPORTED
                    }
                }
            }
            // Conditional / mapped / indexed / typeof / template / import /
            // infer / constructor / this — unmodeled in v0.
            _ => TypeTable::UNSUPPORTED,
        }
    }

    fn lower_literal(&mut self, lit: &TSLiteral<'_>) -> TypeId {
        match lit {
            TSLiteral::BooleanLiteral(b) => self.sink.push(Type::BooleanLiteral(b.value)),
            TSLiteral::NumericLiteral(n) => self.sink.push(Type::NumberLiteral(n.value)),
            TSLiteral::StringLiteral(s) => {
                self.sink.push(Type::StringLiteral(s.value.as_str().into()))
            }
            TSLiteral::BigIntLiteral(b) => {
                self.sink.push(Type::BigIntLiteral(b.value.as_str().into()))
            }
            TSLiteral::TemplateLiteral(t) => {
                if t.expressions.is_empty() && t.quasis.len() == 1 {
                    let text = t.quasis[0].value.cooked.as_deref().unwrap_or("");
                    self.sink.push(Type::StringLiteral(text.into()))
                } else {
                    TypeTable::UNSUPPORTED
                }
            }
            TSLiteral::UnaryExpression(unary) => {
                use oxc_ast::ast::{Expression, UnaryOperator};
                if unary.operator == UnaryOperator::UnaryNegation
                    && let Expression::NumericLiteral(n) = &unary.argument
                {
                    return self.sink.push(Type::NumberLiteral(-n.value));
                }
                TypeTable::UNSUPPORTED
            }
        }
    }

    fn lower_tuple(&mut self, elements: &[TSTupleElement<'_>]) -> TypeId {
        let mut tys = Vec::with_capacity(elements.len());
        for el in elements {
            match el {
                TSTupleElement::TSOptionalType(optional) => {
                    let inner = self.lower_type(&optional.type_annotation);
                    tys.push(self.sink.push(Type::OptionalElem(inner)));
                }
                TSTupleElement::TSRestType(rest) => {
                    let inner = self.lower_type(&rest.type_annotation);
                    tys.push(self.sink.push(Type::RestElem(inner)));
                }
                _ => {
                    let ty = el.to_ts_type();
                    if let TSType::TSNamedTupleMember(named) = ty {
                        let inner = self.lower_tuple_member_type(&named.element_type);
                        if named.optional {
                            tys.push(self.sink.push(Type::OptionalElem(inner)));
                        } else {
                            tys.push(inner);
                        }
                    } else {
                        tys.push(self.lower_type(ty));
                    }
                }
            }
        }
        self.sink.push(Type::Tuple(tys.into_boxed_slice()))
    }

    fn lower_tuple_member_type(&mut self, el: &TSTupleElement<'_>) -> TypeId {
        match el {
            TSTupleElement::TSOptionalType(optional) => self.lower_type(&optional.type_annotation),
            TSTupleElement::TSRestType(rest) => {
                let inner = self.lower_type(&rest.type_annotation);
                self.sink.push(Type::RestElem(inner))
            }
            _ => self.lower_type(el.to_ts_type()),
        }
    }

    fn lower_reference(
        &mut self,
        name: &TSTypeName<'_>,
        type_arguments: Option<&oxc_ast::ast::TSTypeParameterInstantiation<'_>>,
    ) -> TypeId {
        let args: Vec<TypeId> = type_arguments
            .map(|params| params.params.iter().map(|t| self.lower_type(t)).collect())
            .unwrap_or_default();

        let target = match name {
            TSTypeName::IdentifierReference(ident) => {
                let name = ident.name.as_str();
                // Enclosing generic declaration's parameters take precedence.
                if let Some(index) = self.type_params.iter().position(|p| &**p == name) {
                    return self.sink.push(Type::TypeParam {
                        index: u16::try_from(index).unwrap_or(u16::MAX),
                        name: name.into(),
                    });
                }
                // `Array<T>` / `ReadonlyArray<T>` are structural in v0.
                if name == "Array" && args.len() <= 1 {
                    let elem = args.first().copied().unwrap_or(TypeTable::ANY);
                    return self.sink.push(Type::Array(elem));
                }
                if name == "ReadonlyArray" && args.len() <= 1 {
                    let elem = args.first().copied().unwrap_or(TypeTable::ANY);
                    let array = self.sink.push(Type::Array(elem));
                    return self.sink.push(Type::Readonly(array));
                }
                self.resolver.resolve(name)
            }
            TSTypeName::QualifiedName(qualified) => match &qualified.left {
                TSTypeName::IdentifierReference(left) => self.resolver.resolve_qualified(
                    left.name.as_str(),
                    qualified.right.name.as_str(),
                    qualified.right.span,
                ),
                // Deeper qualified names (a.b.c) are unmodeled in v0.
                _ => NameTarget::Ref(RefTarget::Unresolved),
            },
            TSTypeName::ThisExpression(_) => return TypeTable::UNSUPPORTED,
        };

        match target {
            NameTarget::Inline(ty) => {
                if args.is_empty() {
                    ty
                } else {
                    TypeTable::UNSUPPORTED
                }
            }
            NameTarget::EnumMember { symbol, index } => {
                self.sink.push(Type::EnumMember { symbol, index })
            }
            NameTarget::Ref(target) => {
                self.sink.push(Type::Ref { target, args: args.into_boxed_slice() })
            }
        }
    }

    /// Lower object-type members (type literals, interface bodies) to a
    /// [`Type::Object`].
    pub fn lower_members(&mut self, members: &[TSSignature<'_>], force_inexact: bool) -> TypeId {
        let mut out = Vec::with_capacity(members.len());
        let mut inexact = force_inexact;
        for member in members {
            match member {
                TSSignature::TSPropertySignature(prop) => {
                    if let Some(name) = property_key_name(&prop.key) {
                        let ty = self.lower_annotation(prop.type_annotation.as_deref());
                        out.push(Member { name, ty, optional: prop.optional });
                    } else {
                        inexact = true;
                    }
                }
                TSSignature::TSMethodSignature(method) => {
                    use oxc_ast::ast::TSMethodSignatureKind;
                    let Some(name) = property_key_name(&method.key) else {
                        inexact = true;
                        continue;
                    };
                    match method.kind {
                        TSMethodSignatureKind::Method => {
                            let shape = self.lower_function_shape(
                                &method.params,
                                method.return_type.as_deref(),
                            );
                            let ty = self.sink.push(Type::Function(Box::new(shape)));
                            out.push(Member { name, ty, optional: method.optional });
                        }
                        TSMethodSignatureKind::Get => {
                            let ty = self.lower_annotation(method.return_type.as_deref());
                            out.push(Member { name, ty, optional: false });
                        }
                        // A lone setter still makes the property exist; its
                        // type is the parameter type.
                        TSMethodSignatureKind::Set => {
                            let ty = method.params.items.first().map_or(TypeTable::ANY, |p| {
                                self.lower_annotation(p.type_annotation.as_deref())
                            });
                            out.push(Member { name, ty, optional: false });
                        }
                    }
                }
                // Index/call/construct signatures widen the shape beyond what
                // v0 models member-by-member.
                TSSignature::TSIndexSignature(_)
                | TSSignature::TSCallSignatureDeclaration(_)
                | TSSignature::TSConstructSignatureDeclaration(_) => inexact = true,
            }
        }
        self.sink.push(Type::Object(ObjectShape { members: out.into_boxed_slice(), inexact }))
    }

    /// Lower an interface declaration body. `extends` makes the shape inexact.
    pub fn lower_interface(&mut self, iface: &TSInterfaceDeclaration<'_>) -> TypeId {
        self.lower_members(&iface.body.body, !iface.extends.is_empty())
    }

    /// Lower formal parameters + return annotation into a function shape.
    pub fn lower_function_shape(
        &mut self,
        params: &FormalParameters<'_>,
        return_type: Option<&TSTypeAnnotation<'_>>,
    ) -> FunctionShape {
        use oxc_ast::ast::BindingPattern;
        let lowered: Vec<Param> = params
            .items
            .iter()
            .enumerate()
            .map(|(i, p)| Param {
                name: match &p.pattern {
                    BindingPattern::BindingIdentifier(id) => id.name.as_str().into(),
                    _ => format!("arg{i}").into_boxed_str(),
                },
                ty: self.lower_annotation(p.type_annotation.as_deref()),
                optional: p.optional || p.initializer.is_some(),
            })
            .collect();
        let ret = self.lower_annotation(return_type);
        FunctionShape { params: lowered.into_boxed_slice(), ret }
    }
}

/// Static name of a property key, when representable.
pub fn property_key_name(key: &PropertyKey<'_>) -> Option<Box<str>> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str().into()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str().into()),
        PropertyKey::NumericLiteral(n) => Some(n.value.to_string().into_boxed_str()),
        _ => None,
    }
}
