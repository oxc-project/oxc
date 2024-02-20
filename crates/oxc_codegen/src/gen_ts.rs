use crate::context::Context;
use crate::{Codegen, Gen, GenExpr};
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_syntax::precedence::Precedence;

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeParameterDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"<");
        p.print_list(&self.params, ctx);
        p.print_str(b">");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeAnnotation<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.type_annotation.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::TSFunctionType(decl) => decl.gen(p, ctx),
            Self::TSConstructorType(_decl) => {
                // TODO: TSConstructorType
            }
            Self::TSArrayType(decl) => {
                decl.element_type.gen(p, ctx);
                p.print_str(b"[]");
            }
            Self::TSTupleType(decl) => {
                p.print_str(b"[");
                p.print_list(&decl.element_types, ctx);
                p.print_str(b"]");
            }
            Self::TSUnionType(decl) => {
                for (index, item) in decl.types.iter().enumerate() {
                    if index != 0 {
                        p.print_soft_space();
                        p.print_str(b"|");
                        p.print_soft_space();
                    }
                    item.gen(p, ctx);
                }
            }
            Self::TSIntersectionType(decl) => {
                for (index, item) in decl.types.iter().enumerate() {
                    if index != 0 {
                        p.print_soft_space();
                        p.print_str(b"&");
                        p.print_soft_space();
                    }
                    item.gen(p, ctx);
                }
            }
            Self::TSConditionalType(decl) => {
                decl.check_type.gen(p, ctx);
                p.print_str(b" extends ");
                decl.extends_type.gen(p, ctx);
                p.print_str(b" ? ");
                decl.true_type.gen(p, ctx);
                p.print_str(b" : ");
                decl.false_type.gen(p, ctx);
            }
            Self::TSInferType(decl) => {
                p.print_str(b"infer ");
                decl.type_parameter.gen(p, ctx);
            }
            Self::TSIndexedAccessType(decl) => {
                decl.object_type.gen(p, ctx);
                p.print_str(b"[");
                decl.index_type.gen(p, ctx);
                p.print_str(b"]");
            }
            Self::TSMappedType(_decl) => {
                // TODO: Implement this
            }
            Self::TSLiteralType(decl) => {
                decl.literal.gen(p, ctx);
            }
            Self::TSImportType(_decl) => {
                // TODO: Implement this
            }
            Self::TSQualifiedName(decl) => {
                decl.left.gen(p, ctx);
                p.print_str(b".");
                decl.right.gen(p, ctx);
            }
            Self::TSAnyKeyword(_decl) => {
                p.print_str(b"any");
            }
            Self::TSBigIntKeyword(_decl) => {
                p.print_str(b"bigint");
            }
            Self::TSBooleanKeyword(_decl) => {
                p.print_str(b"boolean");
            }
            Self::TSNeverKeyword(_decl) => {
                p.print_str(b"never");
            }
            Self::TSNullKeyword(_decl) => {
                p.print_str(b"null");
            }
            Self::TSNumberKeyword(_decl) => {
                p.print_str(b"number");
            }
            Self::TSObjectKeyword(_decl) => {
                p.print_str(b"object");
            }
            Self::TSStringKeyword(_decl) => {
                p.print_str(b"string");
            }
            Self::TSSymbolKeyword(_decl) => {
                // TODO: TSSymbolKeyword
            }
            Self::TSThisKeyword(_decl) => {
                p.print_str(b"this");
            }
            Self::TSUndefinedKeyword(_decl) => {
                p.print_str(b"undefined");
            }
            Self::TSUnknownKeyword(_decl) => {
                p.print_str(b"unknown");
            }
            Self::TSVoidKeyword(_decl) => {
                p.print_str(b"void");
            }
            Self::TSTemplateLiteralType(_decl) => {
                // TODO: TSTemplateLiteralType
            }
            Self::TSTypeLiteral(decl) => {
                p.print_str(b"{");
                for item in &decl.members {
                    item.gen(p, ctx);
                    p.print_semicolon();
                }
                p.print_soft_space();
                p.print_str(b"}");
            }
            Self::TSTypeOperatorType(decl) => {
                match decl.operator {
                    TSTypeOperator::Keyof => {
                        p.print_str(b"keyof ");
                    }
                    TSTypeOperator::Unique => {
                        p.print_str(b"unique ");
                    }
                    TSTypeOperator::Readonly => {
                        p.print_str(b"readonly ");
                    }
                }
                decl.type_annotation.gen(p, ctx);
            }
            Self::TSTypePredicate(decl) => {
                if decl.asserts {
                    p.print_str(b"asserts ");
                }
                match &decl.parameter_name {
                    TSTypePredicateName::Identifier(ident) => {
                        ident.gen(p, ctx);
                    }
                    TSTypePredicateName::This(_ident) => {
                        p.print_str(b"this");
                    }
                }
                if let Some(type_annotation) = &decl.type_annotation {
                    p.print_str(b" is ");
                    type_annotation.gen(p, ctx);
                }
            }
            Self::TSTypeQuery(decl) => decl.gen(p, ctx),
            Self::TSTypeReference(decl) => {
                decl.type_name.gen(p, ctx);
                if let Some(type_parameters) = &decl.type_parameters {
                    type_parameters.gen(p, ctx);
                }
            }
            Self::JSDocNullableType(decl) => {
                if decl.postfix {
                    decl.type_annotation.gen(p, ctx);
                    p.print_str(b"?");
                } else {
                    p.print_str(b"?");
                    decl.type_annotation.gen(p, ctx);
                }
            }
            Self::JSDocUnknownType(_decl) => p.print_str(b"unknown"),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeName<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::IdentifierReference(decl) => {
                p.print_str(decl.name.as_bytes());
            }
            Self::QualifiedName(decl) => {
                decl.left.gen(p, ctx);
                p.print_str(b".");
                decl.right.gen(p, ctx);
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSLiteral<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::BooleanLiteral(decl) => {
                p.print_str(if decl.value { b"true" } else { b"false" });
            }
            Self::NullLiteral(_decl) => {
                p.print_str(b"null");
            }
            Self::NumberLiteral(decl) => {
                decl.gen(p, ctx);
            }
            Self::BigintLiteral(_decl) => {
                // TODO: BigintLiteral
            }
            Self::RegExpLiteral(_decl) => {
                // TODO: RegExpLiteral
            }
            Self::StringLiteral(decl) => {
                p.print(b'\'');
                p.print_str(decl.value.as_bytes());
                p.print(b'\'');
            }
            Self::TemplateLiteral(_decl) => {
                // TODO: Implement this
            }
            Self::UnaryExpression(_decl) => {
                // TODO: Implement this
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeParameter<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.name.gen(p, ctx);
        if let Some(constraint) = &self.constraint {
            p.print_str(b" extends ");
            constraint.gen(p, ctx);
        }
        if let Some(default) = &self.default {
            p.print_str(b" = ");
            default.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSFunctionType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
        p.print_str(b"(");
        if let Some(this_param) = &self.this_param {
            this_param.this.gen(p, ctx);
            p.print_str(b":");
            if let Some(type_annotation) = &this_param.type_annotation {
                type_annotation.gen(p, ctx);
            } else {
                p.print_str(b"unknown");
            }
            if !self.params.is_empty() {
                p.print_str(b",");
            }
            p.print_soft_space();
        }
        self.params.gen(p, ctx);
        p.print_str(b")");
        p.print_soft_space();
        p.print_str(b"=>");
        p.print_soft_space();
        self.return_type.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSSignature<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_soft_space();
        match self {
            Self::TSIndexSignature(signature) => signature.gen(p, ctx),
            Self::TSPropertySignature(signature) => {
                if signature.readonly {
                    p.print_str(b"readonly");
                    p.print_hard_space();
                }
                if signature.computed {
                    p.print(b'[');
                    signature.key.gen(p, ctx);
                    p.print(b']');
                } else {
                    match &signature.key {
                        PropertyKey::Identifier(key) => {
                            key.gen(p, ctx);
                        }
                        PropertyKey::PrivateIdentifier(key) => {
                            p.print_str(key.name.as_bytes());
                        }
                        PropertyKey::Expression(key) => {
                            key.gen_expr(p, Precedence::Assign, ctx);
                        }
                    }
                }
                if signature.optional {
                    p.print_str(b"?");
                }
                p.print_colon();
                p.print_soft_space();
                if let Some(type_annotation) = &signature.type_annotation {
                    type_annotation.gen(p, ctx);
                }
            }
            Self::TSCallSignatureDeclaration(_signature) => {
                // TODO: TSCallSignatureDeclaration
            }
            Self::TSConstructSignatureDeclaration(_signature) => {
                // TODO: TSConstructSignatureDeclaration
            }
            Self::TSMethodSignature(_signature) => {
                // TODO: TSMethodSignature
            }
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeQuery<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"typeof ");
        self.expr_name.gen(p, ctx);
        if let Some(type_params) = &self.type_parameters {
            type_params.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTypeParameterInstantiation<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"<");
        p.print_list(&self.params, ctx);
        p.print_str(b">");
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSIndexSignature<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"[");
        for (index, parameter) in self.parameters.iter().enumerate() {
            if index != 0 {
                p.print_str(b" | ");
            }
            p.print_str(parameter.name.as_bytes());
            p.print_colon();
            p.print_soft_space();
            parameter.type_annotation.gen(p, ctx);
        }
        p.print_str(b"]");
        p.print_colon();
        p.print_soft_space();
        self.type_annotation.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTupleElement<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            TSTupleElement::TSType(ts_type) => {
                ts_type.gen(p, ctx);
            }
            TSTupleElement::TSOptionalType(ts_type) => {
                ts_type.type_annotation.gen(p, ctx);
                p.print_str(b"?");
            }
            TSTupleElement::TSRestType(ts_type) => {
                p.print_str(b"...");
                ts_type.type_annotation.gen(p, ctx);
            }
            TSTupleElement::TSNamedTupleMember(ts_type) => {
                ts_type.label.gen(p, ctx);
                p.print_str(b":");
                p.print_soft_space();
                ts_type.element_type.gen(p, ctx);
            }
        }
    }
}
