import {camelToSnake, snakeToCamel} from './utils.mjs';

export default function generateAncestorsCode(types) {
    const variantNamesForEnums = Object.create(null);
    let ancestorEnumVariants = '',
        ancestorTypeEnumVariants = '',
        isFunctions = '',
        ancestorTypes = '',
        discriminant = 1;
    for (const type of Object.values(types)) {
        if (type.kind === 'enum') continue;

        const typeSnakeName = camelToSnake(type.name),
            typeScreamingName = typeSnakeName.toUpperCase();
        let offsetCode = '';
        for (const field of type.fields) {
            const offsetVarName = `OFFSET_${typeScreamingName}_${field.name.toUpperCase()}`;
            field.offsetVarName = offsetVarName;
            offsetCode += `pub(crate) const ${offsetVarName}: usize = `
                + `offset_of!(${type.name}, ${field.rawName});\n`;
        }

        const variantNames = [];
        let thisAncestorTypes = '';
        for (const field of type.fields) {
            const fieldTypeName = field.innerTypeName,
                fieldType = types[fieldTypeName];
            if (!fieldType) continue;

            let methodsCode = '';
            for (const otherField of type.fields) {
                if (otherField === field) continue;

                methodsCode += `
                    #[inline]
                    pub fn ${otherField.rawName}(&self) -> &${otherField.rawTypeName} {
                        unsafe {
                            &*(
                                (self.0 as *const u8).add(${otherField.offsetVarName})
                                as *const ${otherField.rawTypeName}
                            )
                        }
                    }
                `;
            }

            const fieldNameCamel = snakeToCamel(field.name),
                lifetime = type.hasLifetime ? "<'a>" : '',
                structName = `${type.name}Without${fieldNameCamel}${lifetime}`;

            thisAncestorTypes += `
                #[repr(transparent)]
                #[derive(Debug)]
                pub struct ${structName}(
                    pub(crate) *const ${type.name}${lifetime}
                );

                impl${lifetime} ${structName} {
                    ${methodsCode}
                }
            `;

            const variantName = `${type.name}${fieldNameCamel}`;
            variantNames.push(variantName);

            ancestorEnumVariants += `${variantName}(${structName}) = ${discriminant},\n`;
            ancestorTypeEnumVariants += `${variantName} = ${discriminant},\n`;
            discriminant++;

            if (fieldType.kind === 'enum') {
                (variantNamesForEnums[fieldTypeName] || (variantNamesForEnums[fieldTypeName] = []))
                    .push(variantName);
            }
        }

        if (variantNames.length > 0) {
            ancestorTypes += `
                ${offsetCode}
                ${thisAncestorTypes}
            `;

            isFunctions += `
                #[inline]
                pub fn is_${typeSnakeName}(&self) -> bool {
                    matches!(self, ${variantNames.map(name => `Self::${name}(_)`).join(' | ')})
                }
            `;
        }
    }

    for (const [typeName, variantNames] of Object.entries(variantNamesForEnums)) {
        isFunctions += `
            #[inline]
            pub fn is_via_${camelToSnake(typeName)}(&self) -> bool {
                matches!(self, ${variantNames.map(name => `Self::${name}(_)`).join(' | ')})
            }
        `;
    }

    const discriminantType = discriminant <= 256 ? 'u8' : 'u16';

    return `
        #![allow(
            unsafe_code,
            clippy::missing_safety_doc,
            clippy::ptr_as_ptr,
            clippy::undocumented_unsafe_blocks,
            clippy::cast_ptr_alignment
        )]

        use memoffset::offset_of;

        use oxc_allocator::{Box, Vec};
        #[allow(clippy::wildcard_imports)]
        use oxc_ast::ast::*;
        use oxc_span::{Atom, SourceType, Span};
        use oxc_syntax::operator::{
            AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
        };

        /// Ancestor type used in AST traversal.
        ///
        /// Encodes both the type of the parent, and child's location in the parent.
        /// i.e. variants for \`BinaryExpressionLeft\` and \`BinaryExpressionRight\`, not just \`BinaryExpression\`.
        //
        // SAFETY: This type MUST be \`#[repr(u8)]\` or \`#[repr(u16)]\` (depending on number of variants)
        // to maintain the safety of \`TraverseCtx::retag_stack\`.
        #[repr(C, ${discriminantType})]
        #[derive(Debug)]
        pub enum Ancestor<'a> {
            None = 0,
            ${ancestorEnumVariants}
        }

        /// Type of [\`Ancestor\`].
        /// Used in [\`crate::TraverseCtx::retag_stack\`].
        // SAFETY: Discriminants of this type must match those for \`Ancestor\` to maintain the safety
        // of \`TraverseCtx::retag_stack\`.
        #[allow(dead_code)]
        #[repr(${discriminantType})]
        pub(crate) enum AncestorType {
            None = 0,
            ${ancestorTypeEnumVariants}
        }

        impl<'a> Ancestor<'a> {
            ${isFunctions}
        }

        ${ancestorTypes}
    `;
}
