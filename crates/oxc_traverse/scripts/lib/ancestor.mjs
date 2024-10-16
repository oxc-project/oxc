import { camelToSnake, snakeToCamel } from './utils.mjs';

/**
 * @param {import('./parse.mjs').Types} types
 */
export default function generateAncestorsCode(types) {
  const variantNamesForEnums = Object.create(null);
  let ancestorTypeEnumVariants = '',
    ancestorEnumVariants = '',
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
      offsetCode += `pub(crate) const ${offsetVarName}: usize = ` +
        `offset_of!(${type.name}, ${field.rawName});\n`;
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
          pub fn ${otherField.rawName}(self) -> &'t ${otherField.rawTypeName} {
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
        lifetimes = type.rawName.length > type.name.length ? `<'a, 't>` : "<'t>",
        structName = `${type.name}Without${fieldNameCamel}${lifetimes}`;

      thisAncestorTypes += `
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug)]
        pub struct ${structName}(
          pub(crate) *const ${type.rawName},
          pub(crate) PhantomData<&'t ()>,
        );

        impl${lifetimes} ${structName} {
          ${methodsCode}
        }
      `;

      const variantName = `${type.name}${fieldNameCamel}`;
      variantNames.push(variantName);

      ancestorTypeEnumVariants += `${variantName} = ${discriminant},\n`;
      ancestorEnumVariants += `${variantName}(${structName}) = AncestorType::${variantName} as u16,\n`;
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
        pub fn is_${typeSnakeName}(self) -> bool {
          matches!(self, ${variantNames.map(name => `Self::${name}(_)`).join(' | ')})
        }
      `;
    }
  }

  for (const [typeName, variantNames] of Object.entries(variantNamesForEnums)) {
    isFunctions += `
      #[inline]
      pub fn is_via_${camelToSnake(typeName)}(self) -> bool {
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

    use std::{cell::Cell, marker::PhantomData};

    use memoffset::offset_of;

    use oxc_allocator::{Box, Vec};
    #[allow(clippy::wildcard_imports)]
    use oxc_ast::ast::*;
    use oxc_syntax::scope::ScopeId;

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
    ///
    /// \`'a\` is lifetime of AST nodes.
    /// \`'t\` is lifetime of the \`Ancestor\` (which inherits lifetime from \`&'t TraverseCtx'\`).
    /// i.e. \`Ancestor\`s can only exist within the body of \`enter_*\` and \`exit_*\` methods
    /// and cannot "escape" from them.
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
    #[derive(Clone, Copy, Debug)]
    pub enum Ancestor<'a, 't> {
      None = AncestorType::None as u16,
      ${ancestorEnumVariants}
    }

    impl<'a, 't> Ancestor<'a, 't> {
      ${isFunctions}
    }

    ${ancestorTypes}
  `;
}
