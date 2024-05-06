import assert from 'assert';
import {camelToSnake, snakeToCamel} from './utils.mjs';

export default function generateAncestorsCode(types) {
    const variantNamesForEnums = Object.create(null);
    let ancestorTypeEnumVariants = '',
        ancestorEnumVariants = '',
        isFunctions = '',
        ancestorTypes = '';
    // Type IDs start at 1, as 0 is reserved for `None`
    let typeId = 1;
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
        let thisAncestorTypes = '',
            fieldId = 0;
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

            ancestorTypeEnumVariants += `${variantName} = ancestor_discriminant(${typeId}, ${fieldId}),\n`;
            ancestorEnumVariants += `${variantName}(${structName}) = AncestorType::${variantName} as u16,\n`;
            fieldId++;

            if (fieldType.kind === 'enum') {
                (variantNamesForEnums[fieldTypeName] || (variantNamesForEnums[fieldTypeName] = []))
                    .push(variantName);
            }
        }

        assert(fieldId <= 256, `Too many fields in ${type.name} to be represented as a u8`);

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

            typeId++;
        }
    }

    assert(typeId <= 256, 'Too many AST types to be represented as a u8');

    for (const [typeName, variantNames] of Object.entries(variantNamesForEnums)) {
        isFunctions += `
            #[inline]
            pub fn is_via_${camelToSnake(typeName)}(&self) -> bool {
                matches!(self, ${variantNames.map(name => `Self::${name}(_)`).join(' | ')})
            }
        `;
    }

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

        /// Generate discriminant for \`Ancestor\` / \`AncestorType\` enums.
        /// There are too many variants to fit in a \`u8\`, so have to use a \`u16\`.
        /// Use that to our advantage by putting type and field each in a single byte.
        /// This makes \`Ancestor::is_*\` methods a single equality operation on the lower byte.
        // TODO: I thought compiler would perform above optimization, but it doesn't.
        const fn ancestor_discriminant(type_id: u8, field_id: u8) -> u16 {
            type_id as u16 + field_id as u16 * 256
        }

        /// Type of [\`Ancestor\`].
        /// Used in [\`crate::TraverseCtx::retag_stack\`].
        #[repr(u16)]
        #[derive(Clone, Copy)]
        #[allow(dead_code)]
        pub(crate) enum AncestorType {
            None = 0,
            ${ancestorTypeEnumVariants}
        }

        /// Ancestor type used in AST traversal.
        ///
        /// Encodes both the type of the parent, and child's location in the parent.
        /// i.e. variants for \`BinaryExpressionLeft\` and \`BinaryExpressionRight\`, not just \`BinaryExpression\`.
        //
        // SAFETY:
        // * This type must be \`#[repr(u16)]\`.
        // * Variant discriminants must correspond to those in \`AncestorType\`.
        //
        // These invariants make it possible to set the discriminant of an \`Ancestor\` without altering
        // the "payload" pointer with:
        // \`*(ancestor as *mut _ as *mut AncestorType) = AncestorType::Program\`.
        // \`TraverseCtx::retag_stack\` uses this technique.
        #[repr(C, u16)]
        #[derive(Debug)]
        pub enum Ancestor<'a> {
            None = AncestorType::None as u16,
            ${ancestorEnumVariants}
        }

        impl<'a> Ancestor<'a> {
            ${isFunctions}
        }

        ${ancestorTypes}
    `;
}
