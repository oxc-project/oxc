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
            Self::TSConstructorType(decl) => {
                decl.gen(p, ctx);
            }
            Self::TSArrayType(decl) => {
                p.print_str(b"(");
                decl.element_type.gen(p, ctx);
                p.print_str(b")[]");
            }
            Self::TSTupleType(decl) => {
                p.print_str(b"[");
                p.print_list(&decl.element_types, ctx);
                p.print_str(b"]");
            }
            Self::TSUnionType(decl) => {
                if decl.types.len() == 1 {
                    decl.types[0].gen(p, ctx);
                    return;
                }

                p.print_str(b"(");
                for (index, item) in decl.types.iter().enumerate() {
                    if index != 0 {
                        p.print_soft_space();
                        p.print_str(b"|");
                        p.print_soft_space();
                    }
                    p.print_str(b"(");
                    item.gen(p, ctx);
                    p.print_str(b")");
                }
                p.print_str(b")");
            }
            Self::TSIntersectionType(decl) => {
                if decl.types.len() == 1 {
                    decl.types[0].gen(p, ctx);
                    return;
                }

                p.print_str(b"(");
                for (index, item) in decl.types.iter().enumerate() {
                    if index != 0 {
                        p.print_soft_space();
                        p.print_str(b"&");
                        p.print_soft_space();
                    }
                    p.print_str(b"(");
                    item.gen(p, ctx);
                    p.print_str(b")");
                }
                p.print_str(b")");
            }
            Self::TSConditionalType(decl) => {
                decl.check_type.gen(p, ctx);
                p.print_str(b" extends (");
                decl.extends_type.gen(p, ctx);
                p.print_str(b") ? ");
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
            Self::TSMappedType(decl) => {
                p.print_str(b"{");
                match decl.readonly {
                    TSMappedTypeModifierOperator::True => {
                        p.print_str(b"readonly");
                    }
                    TSMappedTypeModifierOperator::Plus => {
                        p.print_str(b"+readonly");
                    }
                    TSMappedTypeModifierOperator::Minus => {
                        p.print_str(b"-readonly");
                    }
                    TSMappedTypeModifierOperator::None => {}
                }
                p.print_hard_space();
                p.print_str(b"[");
                decl.type_parameter.name.gen(p, ctx);
                if let Some(constraint) = &decl.type_parameter.constraint {
                    p.print_str(b" in ");
                    constraint.gen(p, ctx);
                }
                if let Some(default) = &decl.type_parameter.default {
                    p.print_str(b" = ");
                    default.gen(p, ctx);
                }
                p.print_str(b"]");
                match decl.optional {
                    TSMappedTypeModifierOperator::True => {
                        p.print_str(b"?");
                    }
                    TSMappedTypeModifierOperator::Plus => {
                        p.print_str(b"+?");
                    }
                    TSMappedTypeModifierOperator::Minus => {
                        p.print_str(b"-?");
                    }
                    TSMappedTypeModifierOperator::None => {}
                }
                p.print_soft_space();
                if let Some(type_annotation) = &decl.type_annotation {
                    p.print_str(b":");
                    p.print_soft_space();
                    type_annotation.gen(p, ctx);
                }
                p.print_semicolon_if_needed();
                p.print_str(b"}");
            }
            Self::TSLiteralType(decl) => {
                decl.literal.gen(p, ctx);
            }
            Self::TSImportType(decl) => {
                if decl.is_type_of {
                    p.print_str(b"typeof ");
                }
                p.print_str(b"import(");
                decl.argument.gen(p, ctx);
                p.print_str(b")");
            }
            Self::TSQualifiedName(decl) => {
                decl.left.gen(p, ctx);
                p.print_str(b".");
                decl.right.gen(p, ctx);
            }
            Self::TSAnyKeyword(_) => {
                p.print_str(b"any");
            }
            Self::TSBigIntKeyword(_) => {
                p.print_str(b"bigint");
            }
            Self::TSBooleanKeyword(_) => {
                p.print_str(b"boolean");
            }
            Self::TSNeverKeyword(_) => {
                p.print_str(b"never");
            }
            Self::TSNullKeyword(_) => {
                p.print_str(b"null");
            }
            Self::TSNumberKeyword(_) => {
                p.print_str(b"number");
            }
            Self::TSObjectKeyword(_) => {
                p.print_str(b"object");
            }
            Self::TSStringKeyword(_) => {
                p.print_str(b"string");
            }
            Self::TSSymbolKeyword(_) => {
                p.print_str(b"symbol");
            }
            Self::TSThisType(_) => {
                p.print_str(b"this");
            }
            Self::TSUndefinedKeyword(_) => {
                p.print_str(b"undefined");
            }
            Self::TSUnknownKeyword(_) => {
                p.print_str(b"unknown");
            }
            Self::TSVoidKeyword(_) => {
                p.print_str(b"void");
            }
            Self::TSTemplateLiteralType(decl) => decl.gen(p, ctx),
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
                    TSTypeOperatorOperator::Keyof => {
                        p.print_str(b"keyof ");
                    }
                    TSTypeOperatorOperator::Unique => {
                        p.print_str(b"unique ");
                    }
                    TSTypeOperatorOperator::Readonly => {
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

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSTemplateLiteralType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        p.print_str(b"`");
        for (index, item) in self.quasis.iter().enumerate() {
            if index != 0 {
                if let Some(types) = self.types.get(index - 1) {
                    p.print_str(b"${");
                    types.gen(p, ctx);
                    p.print_str(b"}");
                }
            }
            p.print_str(item.value.raw.as_bytes());
        }
        p.print_str(b"`");
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
            Self::BooleanLiteral(decl) => decl.gen(p, ctx),
            Self::NullLiteral(decl) => decl.gen(p, ctx),
            Self::NumericLiteral(decl) => decl.gen(p, ctx),
            Self::BigintLiteral(decl) => decl.gen(p, ctx),
            Self::RegExpLiteral(decl) => decl.gen(p, ctx),
            Self::StringLiteral(decl) => decl.gen(p, ctx),
            Self::TemplateLiteral(decl) => decl.gen(p, ctx),
            Self::UnaryExpression(decl) => decl.gen_expr(p, Precedence::Assign, ctx),
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
            if !self.params.is_empty() || self.params.rest.is_some() {
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
                if let Some(type_annotation) = &signature.type_annotation {
                    p.print_colon();
                    p.print_soft_space();
                    type_annotation.gen(p, ctx);
                }
            }
            Self::TSCallSignatureDeclaration(signature) => {
                p.print_str(b"(");
                signature.params.gen(p, ctx);
                p.print_str(b")");
                if let Some(return_type) = &signature.return_type {
                    p.print_colon();
                    p.print_soft_space();
                    return_type.gen(p, ctx);
                }
            }
            Self::TSConstructSignatureDeclaration(signature) => {
                p.print_str(b"new ");
                p.print_str(b"(");
                signature.params.gen(p, ctx);
                p.print_str(b")");
                if let Some(return_type) = &signature.return_type {
                    p.print_colon();
                    p.print_soft_space();
                    return_type.gen(p, ctx);
                }
            }
            Self::TSMethodSignature(signature) => {
                match signature.kind {
                    TSMethodSignatureKind::Method => {}
                    TSMethodSignatureKind::Get => p.print_str(b"get "),
                    TSMethodSignatureKind::Set => p.print_str(b"set "),
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
                p.print_str(b"(");
                signature.params.gen(p, ctx);
                p.print_str(b")");
                if let Some(return_type) = &signature.return_type {
                    p.print_colon();
                    p.print_soft_space();
                    return_type.gen(p, ctx);
                }
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
        if !p.options.enable_typescript {
            return;
        }
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

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSModuleDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.modifiers.contains(ModifierKind::Export) {
            p.print_str(b"export ");
        }
        if self.modifiers.contains(ModifierKind::Declare) {
            p.print_str(b"declare ");
        }
        p.print_str(b"module");
        p.print_space_before_identifier();
        let name = self.id.name();
        p.wrap_quote(name, |p, _| p.print_str(name.as_bytes()));
        p.print_hard_space();
        match &self.body {
            TSModuleDeclarationBody::TSModuleDeclaration(body) => {
                p.print_block_start();
                body.gen(p, ctx);
                p.print_block_end();
            }
            TSModuleDeclarationBody::TSModuleBlock(body) => {
                p.print_block_start();
                for item in &body.body {
                    p.print_semicolon_if_needed();
                    item.gen(p, ctx);
                }
                p.print_semicolon_if_needed();
                p.print_block_end();
            }
        }
        if MINIFY {
            p.print_semicolon();
        }
        p.print_hard_space();
        p.print_soft_newline();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSInterfaceDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if !p.options.enable_typescript {
            return;
        }

        p.print_str(b"interface");
        p.print_hard_space();
        self.id.gen(p, ctx);
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
        if let Some(extends) = &self.extends {
            if !extends.is_empty() {
                p.print_str(b" extends ");
                p.print_list(extends, ctx);
            }
        }
        p.print_soft_space();
        p.print_block_start();
        for item in &self.body.body {
            p.print_indent();
            p.print_semicolon_if_needed();
            item.gen(p, ctx);
            p.print_semicolon_after_statement();
        }
        p.print_block_end();
        if MINIFY {
            p.print_hard_space();
        }
        p.print_soft_newline();
        p.needs_semicolon = false;
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSInterfaceHeritage<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        self.expression.gen_expr(p, Precedence::Call, ctx);
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSEnumDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if !p.options.enable_typescript {
            return;
        }

        p.print_indent();
        if self.modifiers.contains(ModifierKind::Export) {
            p.print_str(b"export ");
        }
        if p.options.enable_typescript && self.modifiers.contains(ModifierKind::Declare) {
            p.print_str(b"declare ");
        }
        if self.modifiers.contains(ModifierKind::Const) {
            p.print_str(b"const ");
        }
        p.print_space_before_identifier();
        p.print_str(b"enum ");
        self.id.gen(p, ctx);
        p.print_space_before_identifier();
        p.print_block_start();
        p.print_list(&self.members, ctx);
        p.print_block_end();
        p.print_hard_space();
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSEnumMember<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match &self.id {
            TSEnumMemberName::Identifier(decl) => decl.gen(p, ctx),
            TSEnumMemberName::StringLiteral(decl) => decl.gen(p, ctx),
            TSEnumMemberName::ComputedPropertyName(decl) => {
                p.print_str(b"[");
                decl.gen_expr(p, Precedence::lowest(), ctx);
                p.print_str(b"]");
            }
            TSEnumMemberName::NumericLiteral(decl) => decl.gen(p, ctx),
        }
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSConstructorType<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if self.r#abstract {
            p.print_str(b"abstract ");
        }
        p.print_str(b"new ");
        if let Some(type_parameters) = &self.type_parameters {
            type_parameters.gen(p, ctx);
        }
        p.print_str(b"(");
        self.params.gen(p, ctx);
        p.print_str(b")");
        p.print_soft_space();
        p.print_str(b"=>");
        p.print_soft_space();
        self.return_type.gen(p, ctx);
    }
}

impl<'a, const MINIFY: bool> Gen<MINIFY> for TSImportEqualsDeclaration<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        if !p.options.enable_typescript {
            return;
        }
        p.print_str(b"import ");
        self.id.gen(p, ctx);
        p.print_str(b" = ");
        self.module_reference.gen(p, ctx);
        p.print_semicolon_after_statement();
    }
}
impl<'a, const MINIFY: bool> Gen<MINIFY> for TSModuleReference<'a> {
    fn gen(&self, p: &mut Codegen<{ MINIFY }>, ctx: Context) {
        match self {
            Self::ExternalModuleReference(decl) => {
                p.print_str(b"require(");
                decl.expression.gen(p, ctx);
                p.print_str(b")");
            }
            Self::TypeName(decl) => {
                decl.gen(p, ctx);
            }
        }
    }
}

impl<'a, const MINIFY: bool> GenExpr<MINIFY> for TSTypeAssertion<'a> {
    fn gen_expr(&self, p: &mut Codegen<{ MINIFY }>, precedence: Precedence, ctx: Context) {
        if p.options.enable_typescript {
            p.print_str(b"<");
            self.type_annotation.gen(p, ctx);
            p.print_str(b">");
        }
        self.expression.gen_expr(p, precedence, ctx);
    }
}
