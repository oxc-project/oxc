use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    array, dynamic_text,
    format::{
        print::{array, object, property, template_literal},
        Format,
    },
    group, hardline, indent,
    ir::{Doc, JoinSeparator},
    join, line, softline, text, wrap, Prettier,
};

impl<'a> Format<'a> for TSTypeAliasDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        if self.declare {
            parts.push(text!("declare "));
        }

        parts.push(text!("type "));
        parts.push(self.id.format(p));

        if let Some(params) = &self.type_parameters {
            parts.push(params.format(p));
        }

        parts.push(text!(" = "));
        parts.push(self.type_annotation.format(p));

        if let Some(semi) = p.semi() {
            parts.push(semi);
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSType::TSAnyKeyword(v) => v.format(p),
            TSType::TSBigIntKeyword(v) => v.format(p),
            TSType::TSBooleanKeyword(v) => v.format(p),
            TSType::TSIntrinsicKeyword(v) => v.format(p),
            TSType::TSNeverKeyword(v) => v.format(p),
            TSType::TSNullKeyword(v) => v.format(p),
            TSType::TSNumberKeyword(v) => v.format(p),
            TSType::TSObjectKeyword(v) => v.format(p),
            TSType::TSStringKeyword(v) => v.format(p),
            TSType::TSSymbolKeyword(v) => v.format(p),
            TSType::TSThisType(v) => v.format(p),
            TSType::TSUndefinedKeyword(v) => v.format(p),
            TSType::TSUnknownKeyword(v) => v.format(p),
            TSType::TSVoidKeyword(v) => v.format(p),
            TSType::TSArrayType(v) => v.format(p),
            TSType::TSConditionalType(v) => v.format(p),
            TSType::TSConstructorType(v) => v.format(p),
            TSType::TSFunctionType(v) => v.format(p),
            TSType::TSImportType(v) => v.format(p),
            TSType::TSIndexedAccessType(v) => v.format(p),
            TSType::TSInferType(v) => v.format(p),
            TSType::TSIntersectionType(v) => v.format(p),
            TSType::TSLiteralType(v) => v.format(p),
            TSType::TSMappedType(v) => v.format(p),
            TSType::TSNamedTupleMember(v) => v.format(p),
            TSType::TSQualifiedName(v) => v.format(p),
            TSType::TSTemplateLiteralType(v) => v.format(p),
            TSType::TSTupleType(v) => v.format(p),
            TSType::TSTypeLiteral(v) => v.format(p),
            TSType::TSTypeOperatorType(v) => v.format(p),
            TSType::TSTypePredicate(v) => v.format(p),
            TSType::TSTypeQuery(v) => v.format(p),
            TSType::TSTypeReference(v) => v.format(p),
            TSType::TSUnionType(v) => v.format(p),
            TSType::TSParenthesizedType(v) => v.format(p),
            TSType::JSDocNullableType(v) => v.format(p),
            TSType::JSDocNonNullableType(v) => v.format(p),
            TSType::JSDocUnknownType(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for TSAnyKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("any")
    }
}

impl<'a> Format<'a> for TSBigIntKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("bigint")
    }
}

impl<'a> Format<'a> for TSBooleanKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("boolean")
    }
}

impl<'a> Format<'a> for TSIntrinsicKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("intrinsic")
    }
}

impl<'a> Format<'a> for TSNeverKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("never")
    }
}

impl<'a> Format<'a> for TSNullKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("null")
    }
}

impl<'a> Format<'a> for TSNumberKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("number")
    }
}

impl<'a> Format<'a> for TSObjectKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("object")
    }
}

impl<'a> Format<'a> for TSStringKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("string")
    }
}

impl<'a> Format<'a> for TSSymbolKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("symbol")
    }
}

impl<'a> Format<'a> for TSThisType {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("this")
    }
}

impl<'a> Format<'a> for TSUndefinedKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("undefined")
    }
}

impl<'a> Format<'a> for TSUnknownKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("unknown")
    }
}

impl<'a> Format<'a> for TSVoidKeyword {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        text!("void")
    }
}

impl<'a> Format<'a> for TSArrayType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let element_type_doc = self.element_type.format(p);
        array!(p, [element_type_doc, text!("[]")])
    }
}

impl<'a> Format<'a> for TSConditionalType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(self.check_type.format(p));
        parts.push(text!(" extends "));
        parts.push(self.extends_type.format(p));
        parts.push(text!(" ? "));
        parts.push(self.true_type.format(p));
        parts.push(text!(" : "));
        parts.push(self.false_type.format(p));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSConstructorType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        if self.r#abstract {
            parts.push(text!("abstract "));
        }
        parts.push(text!("new "));
        parts.push(self.params.format(p));
        let type_annotation_doc = self.return_type.type_annotation.format(p);
        parts.push(array!(p, [text!(" => "), type_annotation_doc]));
        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSFunctionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        parts.push(self.params.format(p));

        parts.push(text!(" => "));
        parts.push(self.return_type.type_annotation.format(p));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSThisParameter<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(text!("this"));

        if let Some(type_annotation) = &self.type_annotation {
            parts.push(text!(": "));
            parts.push(type_annotation.type_annotation.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSImportType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        if self.is_type_of {
            parts.push(text!("typeof "));
        }

        parts.push(text!("import("));
        parts.push(self.parameter.format(p));
        // ToDo: attributes
        parts.push(text!(")"));

        if let Some(qualifier) = &self.qualifier {
            parts.push(text!("."));
            parts.push(qualifier.format(p));
        }

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSIndexedAccessType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(self.object_type.format(p));
        parts.push(text!("["));
        parts.push(self.index_type.format(p));
        parts.push(text!("]"));
        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSInferType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let type_parameter_doc = self.type_parameter.format(p);
        array!(p, [text!("infer "), type_parameter_doc])
    }
}

impl<'a> Format<'a> for TSIntersectionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        let mut add_symbol = false;

        for ts_type in &self.types {
            if add_symbol {
                parts.push(text!(" & "));
            } else {
                add_symbol = true;
            }

            parts.push(ts_type.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSLiteralType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match &self.literal {
            TSLiteral::BooleanLiteral(v) => v.format(p),
            TSLiteral::NullLiteral(v) => v.format(p),
            TSLiteral::NumericLiteral(v) => v.format(p),
            TSLiteral::BigIntLiteral(v) => v.format(p),
            TSLiteral::RegExpLiteral(v) => v.format(p),
            TSLiteral::StringLiteral(v) => v.format(p),
            TSLiteral::TemplateLiteral(v) => v.format(p),
            TSLiteral::UnaryExpression(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for TSMappedType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts: Vec<'_, Doc<'_>> = Vec::new_in(p.allocator);

        match self.readonly {
            TSMappedTypeModifierOperator::Plus => parts.push(text!("+readonly ")),
            TSMappedTypeModifierOperator::Minus => parts.push(text!("-readonly ")),
            TSMappedTypeModifierOperator::True => parts.push(text!("readonly ")),
            TSMappedTypeModifierOperator::None => (),
        }

        parts.push(text!("["));
        parts.push(self.type_parameter.format(p));

        if let Some(name_type) = &self.name_type {
            parts.push(text!(" as "));
            parts.push(name_type.format(p));
        }

        parts.push(text!("]"));

        match self.optional {
            TSMappedTypeModifierOperator::Plus => parts.push(text!("+?")),
            TSMappedTypeModifierOperator::Minus => parts.push(text!("-?")),
            TSMappedTypeModifierOperator::True => parts.push(text!("?")),
            TSMappedTypeModifierOperator::None => (),
        }

        if let Some(type_annotation) = &self.type_annotation {
            parts.push(text!(": "));
            parts.push(type_annotation.format(p));
        }

        array!(
            p,
            [
                text!("{ "),
                // TODO: check ident/grouping in method/method-signature.ts
                group!(p, parts),
                text!(" }")
            ]
        )
    }
}

impl<'a> Format<'a> for TSNamedTupleMember<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(self.label.format(p));

        if self.optional {
            parts.push(text!("?"));
        }

        parts.push(text!(": "));
        parts.push(self.element_type.format(p));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSRestType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let type_annotation_doc = self.type_annotation.format(p);
        array!(p, [text!("..."), type_annotation_doc])
    }
}

impl<'a> Format<'a> for TSOptionalType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let type_annotation_doc = self.type_annotation.format(p);
        array!(p, [type_annotation_doc, text!("?")])
    }
}

impl<'a> Format<'a> for TSQualifiedName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let left_doc = self.left.format(p);
        let right_doc = self.right.format(p);
        array!(p, [left_doc, text!("."), right_doc])
    }
}

impl<'a> Format<'a> for TSTemplateLiteralType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        template_literal::print_template_literal(
            p,
            &template_literal::TemplateLiteralLike::TSTemplateLiteralType(self),
        )
    }
}

impl<'a> Format<'a> for TSTupleType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        array::print_array(p, &array::ArrayLike::TSTupleType(self))
    }
}

impl<'a> Format<'a> for TSTypeLiteral<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TSTypeLiteral, {
            object::print_object(p, object::ObjectLike::TSTypeLiteral(self))
        })
    }
}

impl<'a> Format<'a> for TSTypeOperator<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(text!(self.operator.to_str()));
        parts.push(text!(" "));
        parts.push(self.type_annotation.format(p));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSTypePredicate<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        if self.asserts {
            parts.push(text!("asserts "));
        }
        parts.push(self.parameter_name.format(p));

        if let Some(type_annotation) = &self.type_annotation {
            parts.push(text!(" is "));
            parts.push(type_annotation.type_annotation.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSTypePredicateName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSTypePredicateName::Identifier(it) => it.format(p),
            TSTypePredicateName::This(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for TSTypeQuery<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(text!("typeof "));

        match &self.expr_name {
            TSTypeQueryExprName::TSImportType(import_type) => parts.push(import_type.format(p)),
            TSTypeQueryExprName::IdentifierReference(identifier_reference) => {
                parts.push(identifier_reference.format(p));
            }
            TSTypeQueryExprName::QualifiedName(qualified_name) => {
                parts.push(qualified_name.format(p));
            }
        }

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSTypeReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(self.type_name.format(p));
        if let Some(params) = &self.type_parameters {
            parts.push(params.format(p));
        }
        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSParenthesizedType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TSParenthesizedType, { self.type_annotation.format(p) })
    }
}

impl<'a> Format<'a> for TSUnionType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        let mut add_symbol = false;

        for ts_type in &self.types {
            if add_symbol {
                parts.push(text!(" | "));
            } else {
                add_symbol = true;
            }

            parts.push(ts_type.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for JSDocNullableType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSDocNonNullableType<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for JSDocUnknownType {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        line!()
    }
}

impl<'a> Format<'a> for TSInterfaceDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TSInterfaceDeclaration, {
            let mut parts = Vec::new_in(p.allocator);

            if self.declare {
                parts.push(text!("declare "));
            }

            parts.push(text!("interface "));
            parts.push(self.id.format(p));

            if let Some(type_parameters) = &self.type_parameters {
                parts.push(type_parameters.format(p));
            }

            parts.push(text!(" "));

            if let Some(extends) = &self.extends {
                if extends.len() > 0 {
                    let mut extends_parts = Vec::new_in(p.allocator);
                    let mut display_comma = false;

                    extends_parts.push(text!("extends "));

                    for extend in extends {
                        if display_comma {
                            extends_parts.push(text!(", "));
                        } else {
                            display_comma = true;
                        }

                        extends_parts.push(extend.expression.format(p));
                        if let Some(type_parameters) = &extend.type_parameters {
                            extends_parts.push(type_parameters.format(p));
                        }
                    }

                    parts.extend(extends_parts);
                    parts.push(text!(" "));
                }
            }

            parts.push(text!("{"));
            if self.body.body.len() > 0 {
                let mut indent_parts = Vec::new_in(p.allocator);
                for sig in &self.body.body {
                    indent_parts.push(hardline!(p));
                    indent_parts.push(sig.format(p));

                    if let Some(semi) = p.semi() {
                        indent_parts.push(semi);
                    }
                }
                parts.push(indent!(p, indent_parts));
                parts.push(hardline!(p));
            }
            parts.push(text!("}"));
            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for TSEnumDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        if self.declare {
            parts.push(text!("declare "));
        }
        if self.r#const {
            parts.push(text!("const "));
        }
        parts.push(text!("enum "));
        parts.push(self.id.format(p));
        parts.push(text!(" {"));
        if self.members.len() > 0 {
            let mut indent_parts = Vec::new_in(p.allocator);
            for member in &self.members {
                indent_parts.push(hardline!(p));
                indent_parts.push(member.format(p));
            }
            parts.push(indent!(p, indent_parts));
            parts.push(hardline!(p));
        }
        parts.push(text!("}"));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSEnumMember<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(self.id.format(p));

        if let Some(initializer) = &self.initializer {
            parts.push(text!(" = "));
            parts.push(initializer.format(p));
        }

        parts.push(text!(","));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSEnumMemberName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSEnumMemberName::Identifier(identifier) => identifier.format(p),
            TSEnumMemberName::String(string_literal) => string_literal.format(p),
        }
    }
}

impl<'a> Format<'a> for TSModuleDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        if self.declare {
            parts.push(text!("declare "));
        }

        parts.push(text!(self.kind.as_str()));
        parts.push(text!(" "));
        parts.push(self.id.format(p));
        parts.push(text!(" {"));

        if let Some(body) = &self.body {
            if !body.is_empty() {
                parts.push(indent!(p, [hardline!(p), body.format(p)]));
                parts.push(hardline!(p));
            }
        }

        parts.push(text!("}"));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSModuleDeclarationName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSModuleDeclarationName::Identifier(identifier) => identifier.format(p),
            TSModuleDeclarationName::StringLiteral(string_literal) => string_literal.format(p),
        }
    }
}

impl<'a> Format<'a> for TSModuleDeclarationBody<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSModuleDeclarationBody::TSModuleBlock(module_block) => module_block.format(p),
            TSModuleDeclarationBody::TSModuleDeclaration(module_declaration) => {
                module_declaration.format(p)
            }
        }
    }
}

impl<'a> Format<'a> for TSModuleBlock<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        let mut add_line = false;

        for body_part in &self.body {
            if add_line {
                parts.push(line!());
            } else {
                add_line = true;
            }

            parts.push(body_part.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSImportEqualsDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(text!("import "));

        if self.import_kind == ImportOrExportKind::Type {
            parts.push(text!("type "));
        }

        parts.push(self.id.format(p));
        parts.push(text!(" = "));
        parts.push(self.module_reference.format(p));

        if let Some(semi) = p.semi() {
            parts.push(semi);
        }
        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSModuleReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSModuleReference::IdentifierReference(it) => it.format(p),
            TSModuleReference::QualifiedName(it) => it.format(p),
            TSModuleReference::ExternalModuleReference(v) => v.format(p),
        }
    }
}

impl<'a> Format<'a> for TSTypeName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSTypeName::IdentifierReference(it) => it.format(p),
            TSTypeName::QualifiedName(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for TSExternalModuleReference<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        array!(p, [text!("require("), expression_doc, text!(")")])
    }
}

impl<'a> Format<'a> for TSTypeParameter<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        if self.r#in {
            parts.push(text!("in "));
        }

        if self.out {
            parts.push(text!("out "));
        }

        parts.push(self.name.format(p));

        if let Some(constraint) = &self.constraint {
            parts.push(text!(" extends "));
            parts.push(constraint.format(p));
        }

        if let Some(default) = &self.default {
            parts.push(text!(" = "));
            parts.push(default.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSTypeParameterDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        let mut print_comma = false;

        parts.push(text!("<"));

        for param in &self.params {
            if print_comma {
                parts.push(text!(", "));
            } else {
                print_comma = true;
            }

            parts.push(param.format(p));
        }

        parts.push(text!(">"));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSTypeParameterInstantiation<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);
        let mut print_comma = false;

        parts.push(text!("<"));

        for param in &self.params {
            if print_comma {
                parts.push(text!(", "));
            } else {
                print_comma = true;
            }

            parts.push(param.format(p));
        }

        parts.push(text!(">"));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSTupleElement<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSTupleElement::TSOptionalType(it) => it.format(p),
            TSTupleElement::TSRestType(it) => it.format(p),
            TSTupleElement::TSAnyKeyword(it) => it.format(p),
            TSTupleElement::TSBigIntKeyword(it) => it.format(p),
            TSTupleElement::TSBooleanKeyword(it) => it.format(p),
            TSTupleElement::TSIntrinsicKeyword(it) => it.format(p),
            TSTupleElement::TSNeverKeyword(it) => it.format(p),
            TSTupleElement::TSNullKeyword(it) => it.format(p),
            TSTupleElement::TSNumberKeyword(it) => it.format(p),
            TSTupleElement::TSObjectKeyword(it) => it.format(p),
            TSTupleElement::TSStringKeyword(it) => it.format(p),
            TSTupleElement::TSSymbolKeyword(it) => it.format(p),
            TSTupleElement::TSUndefinedKeyword(it) => it.format(p),
            TSTupleElement::TSUnknownKeyword(it) => it.format(p),
            TSTupleElement::TSVoidKeyword(it) => it.format(p),
            TSTupleElement::TSArrayType(it) => it.format(p),
            TSTupleElement::TSConditionalType(it) => it.format(p),
            TSTupleElement::TSConstructorType(it) => it.format(p),
            TSTupleElement::TSFunctionType(it) => it.format(p),
            TSTupleElement::TSImportType(it) => it.format(p),
            TSTupleElement::TSIndexedAccessType(it) => it.format(p),
            TSTupleElement::TSInferType(it) => it.format(p),
            TSTupleElement::TSIntersectionType(it) => it.format(p),
            TSTupleElement::TSLiteralType(it) => it.format(p),
            TSTupleElement::TSMappedType(it) => it.format(p),
            TSTupleElement::TSNamedTupleMember(it) => it.format(p),
            TSTupleElement::TSQualifiedName(it) => it.format(p),
            TSTupleElement::TSTemplateLiteralType(it) => it.format(p),
            TSTupleElement::TSThisType(it) => it.format(p),
            TSTupleElement::TSTupleType(it) => it.format(p),
            TSTupleElement::TSTypeLiteral(it) => it.format(p),
            TSTupleElement::TSTypeOperatorType(it) => it.format(p),
            TSTupleElement::TSTypePredicate(it) => it.format(p),
            TSTupleElement::TSTypeQuery(it) => it.format(p),
            TSTupleElement::TSTypeReference(it) => it.format(p),
            TSTupleElement::TSUnionType(it) => it.format(p),
            TSTupleElement::TSParenthesizedType(it) => it.format(p),
            TSTupleElement::JSDocNullableType(it) => it.format(p),
            TSTupleElement::JSDocNonNullableType(it) => it.format(p),
            TSTupleElement::JSDocUnknownType(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for TSExportAssignment<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TSExportAssignment, {
            let mut parts = Vec::new_in(p.allocator);

            parts.push(text!("export = "));
            parts.push(self.expression.format(p));
            if let Some(semi) = p.semi() {
                parts.push(semi);
            }

            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for TSNamespaceExportDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        // wrap!(p, self, TSNamespaceExportDeclaration, {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(text!("export as namespace "));
        parts.push(self.id.format(p));
        if let Some(semi) = p.semi() {
            parts.push(semi);
        }

        array!(p, parts)
        // })
    }
}

impl<'a> Format<'a> for TSClassImplements<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(self.expression.format(p));

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSTypeAssertion<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let type_annotation_doc = self.type_annotation.format(p);
        let expression_doc = self.expression.format(p);
        array!(p, [text!("<"), type_annotation_doc, text!(">"), expression_doc])
    }
}

impl<'a> Format<'a> for TSSatisfiesExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        let type_annotation_doc = self.type_annotation.format(p);
        array!(p, [expression_doc, text!(" satisfies "), type_annotation_doc])
    }
}

impl<'a> Format<'a> for TSInstantiationExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        let type_parameters_doc = self.type_parameters.format(p);
        array!(p, [expression_doc, type_parameters_doc])
    }
}

impl<'a> Format<'a> for TSNonNullExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        array!(p, [expression_doc, text!("!")])
    }
}

impl<'a> Format<'a> for TSIndexSignature<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        if self.readonly {
            parts.push(text!("readonly "));
        }

        parts.push(text!("["));
        for param in &self.parameters {
            parts.push(param.format(p));
        }
        parts.push(text!("]: "));
        parts.push(self.type_annotation.type_annotation.format(p));

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSIndexSignatureName<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let type_annotation_doc = self.type_annotation.type_annotation.format(p);
        array!(p, [dynamic_text!(p, self.name.as_str()), text!(": "), type_annotation_doc])
    }
}

impl<'a> Format<'a> for TSPropertySignature<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        wrap!(p, self, TSPropertySignature, {
            let mut parts = Vec::new_in(p.allocator);
            if self.readonly {
                parts.push(text!("readonly "));
            }
            let key_doc = property::print_property_key(
                p,
                &property::PropertyKeyLike::PropertyKey(&self.key),
                self.computed,
            );
            parts.push(key_doc);
            if let Some(ty) = &self.type_annotation {
                if self.optional {
                    parts.push(text!("?"));
                }
                parts.push(text!(":"));
                parts.push(text!(" "));
                parts.push(ty.type_annotation.format(p));
            }
            array!(p, parts)
        })
    }
}

impl<'a> Format<'a> for TSCallSignatureDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        parts.push(self.params.format(p));

        if let Some(return_type) = &self.return_type {
            parts.push(text!(": "));
            parts.push(return_type.type_annotation.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSConstructSignatureDeclaration<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        parts.push(text!("new "));

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        parts.push(self.params.format(p));

        if let Some(return_type) = &self.return_type {
            parts.push(text!(": "));
            parts.push(return_type.type_annotation.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSMethodSignature<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let mut parts = Vec::new_in(p.allocator);

        let key_doc = if self.computed {
            array!(p, [text!("["), self.key.format(p), text!("]")])
        } else {
            self.key.format(p)
        };
        parts.push(key_doc);

        if self.optional {
            parts.push(text!("?"));
        }

        if let Some(type_parameters) = &self.type_parameters {
            parts.push(type_parameters.format(p));
        }

        parts.push(self.params.format(p));

        if let Some(return_type) = &self.return_type {
            parts.push(text!(": "));
            parts.push(return_type.type_annotation.format(p));
        }

        array!(p, parts)
    }
}

impl<'a> Format<'a> for TSSignature<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            TSSignature::TSIndexSignature(it) => it.format(p),
            TSSignature::TSPropertySignature(it) => it.format(p),
            TSSignature::TSCallSignatureDeclaration(it) => it.format(p),
            TSSignature::TSConstructSignatureDeclaration(it) => it.format(p),
            TSSignature::TSMethodSignature(it) => it.format(p),
        }
    }
}

impl<'a> Format<'a> for TSAsExpression<'a> {
    fn format(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        let expression_doc = self.expression.format(p);
        let type_annotation_doc = self.type_annotation.format(p);
        array!(p, [expression_doc, text!(" as "), type_annotation_doc])
    }
}
