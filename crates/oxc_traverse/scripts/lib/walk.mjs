import assert from 'assert';
import { camelToSnake, snakeToCamel } from './utils.mjs';

/**
 * @typedef {import('./parse.mjs').Types} Types
 * @typedef {import('./parse.mjs').StructType} StructType
 * @typedef {import('./parse.mjs').EnumType} EnumType
 * @typedef {import('./parse.mjs').Field} Field
 */

/**
 * @param {Types} types
 */
export default function generateWalkFunctionsCode(types) {
  let walkMethods = '';
  for (const type of Object.values(types)) {
    if (type.kind === 'struct') {
      walkMethods += generateWalkForStruct(type, types);
    } else {
      walkMethods += generateWalkForEnum(type, types);
    }
  }

  return `
    #![allow(
      unsafe_code,
      clippy::missing_safety_doc,
      clippy::missing_panics_doc,
      clippy::undocumented_unsafe_blocks,
      clippy::semicolon_if_nothing_returned,
      clippy::ptr_as_ptr,
      clippy::ref_as_ptr,
      clippy::borrow_as_ptr,
      clippy::cast_ptr_alignment
    )]

    use std::{cell::Cell, marker::PhantomData};

    use oxc_allocator::Vec;
    #[allow(clippy::wildcard_imports)]
    use oxc_ast::ast::*;
    use oxc_syntax::scope::ScopeId;

    use crate::{ancestor::{self, AncestorType}, Ancestor, Traverse, TraverseCtx};

    ${walkMethods}

    pub(crate) unsafe fn walk_statements<'a, Tr: Traverse<'a>>(
      traverser: &mut Tr,
      stmts: *mut Vec<'a, Statement<'a>>,
      ctx: &mut TraverseCtx<'a>
    ) {
      traverser.enter_statements(&mut *stmts, ctx);
      for stmt in (*stmts).iter_mut() {
        walk_statement(traverser, stmt, ctx);
      }
      traverser.exit_statements(&mut *stmts, ctx);
    }
  `;
}

/**
 * @param {StructType} type
 * @param {Types} types
 */
function generateWalkForStruct(type, types) {
  /** @type {Field | undefined} */
  let scopeIdField;
  const visitedFields = type.fields.filter(field => {
    if (field.name === 'scope_id' && field.typeName === `Cell<Option<ScopeId>>`) {
      scopeIdField = field;
    }
    return field.innerTypeName in types;
  });

  const { scopeArgs } = type;
  /** @type {Field | undefined} */
  let scopeEnterField,
    /** @type {Field | undefined} */
    scopeExitField;
  let enterScopeCode = '', exitScopeCode = '';

  if (scopeArgs && scopeIdField) {
    // Get field to enter scope before
    const enterFieldName = scopeArgs.enterScopeBefore;
    if (enterFieldName) {
      scopeEnterField = visitedFields.find(field => field.name === enterFieldName);
      assert(
        scopeEnterField,
        `\`ast\` attr says to enter scope before field '${enterFieldName}' ` +
          `in '${type.name}', but that field is not visited`,
      );
    } else {
      scopeEnterField = visitedFields[0];
    }

    // Get field to exit scope before
    const exitFieldName = scopeArgs.exitScopeBefore;
    if (exitFieldName) {
      scopeExitField = visitedFields.find(field => field.name === exitFieldName);
      assert(
        scopeExitField,
        `\`ast\` attr says to exit scope before field '${exitFieldName}' ` +
          `in '${type.name}', but that field is not visited`,
      );
    }

    // TODO: Maybe this isn't quite right. `scope_id` fields are `Cell<Option<ScopeId>>`,
    // so visitor is able to alter the `scope_id` of a node from higher up the tree,
    // but we don't take that into account.
    // Visitor should not do that though, so maybe it's OK.
    // In final version, we should not make `scope_id` fields `Cell`s to prevent this.
    enterScopeCode = `
      let previous_scope_id = ctx.current_scope_id();
      ctx.set_current_scope_id((*(${makeFieldCode(scopeIdField)})).get().unwrap());
    `;
    exitScopeCode = `ctx.set_current_scope_id(previous_scope_id);`;
  }

  const fieldsCodes = visitedFields.map((field, index) => {
    const fieldWalkName = `walk_${camelToSnake(field.innerTypeName)}`,
      fieldCamelName = snakeToCamel(field.name);
    const scopeCode = field === scopeEnterField
      ? enterScopeCode
      : field === scopeExitField
      ? exitScopeCode
      : '';

    let tagCode = '', retagCode = '';
    if (index === 0) {
      tagCode = `
        let pop_token = ctx.push_stack(
            Ancestor::${type.name}${fieldCamelName}(
                ancestor::${type.name}Without${fieldCamelName}(node, PhantomData)
            )
        );
      `;
    } else {
      retagCode = `ctx.retag_stack(AncestorType::${type.name}${fieldCamelName});`;
    }

    const fieldCode = makeFieldCode(field);

    if (field.wrappers[0] === 'Option') {
      let walkCode;
      if (field.wrappers.length === 2 && field.wrappers[1] === 'Vec') {
        if (field.typeNameInner === 'Statement') {
          // Special case for `Option<Vec<Statement>>`
          walkCode = `walk_statements(traverser, field as *mut _, ctx);`;
        } else {
          walkCode = `
            for item in field.iter_mut() {
              ${fieldWalkName}(traverser, item as *mut _, ctx);
            }
          `.trim();
        }
      } else if (field.wrappers.length === 2 && field.wrappers[1] === 'Box') {
        walkCode = `${fieldWalkName}(traverser, (&mut **field) as *mut _, ctx);`;
      } else {
        assert(field.wrappers.length === 1, `Cannot handle struct field with type ${field.typeName}`);
        walkCode = `${fieldWalkName}(traverser, field as *mut _, ctx);`;
      }

      return `
        ${scopeCode}
        ${tagCode}
        if let Some(field) = &mut *(${fieldCode}) {
          ${retagCode}
          ${walkCode}
        }
      `;
    }

    if (field.wrappers[0] === 'Vec') {
      let walkVecCode;
      if (field.wrappers.length === 1 && field.innerTypeName === 'Statement') {
        // Special case for `Vec<Statement>`
        walkVecCode = `walk_statements(traverser, ${fieldCode}, ctx);`;
      } else {
        let walkCode = `${fieldWalkName}(traverser, item as *mut _, ctx);`,
          iterModifier = '';
        if (field.wrappers.length === 2 && field.wrappers[1] === 'Option') {
          iterModifier = '.flatten()';
        } else {
          assert(
            field.wrappers.length === 1,
            `Cannot handle struct field with type ${field.type}`,
          );
        }
        walkVecCode = `
          for item in (*(${fieldCode})).iter_mut()${iterModifier} {
            ${walkCode}
          }
        `.trim();
      }

      return `
        ${scopeCode}
        ${tagCode || retagCode}
        ${walkVecCode}
      `;
    }

    if (field.wrappers.length === 1 && field.wrappers[0] === 'Box') {
      return `
        ${scopeCode}
        ${tagCode || retagCode}
        ${fieldWalkName}(traverser, (&mut **(${fieldCode})) as *mut _, ctx);
      `;
    }

    assert(field.wrappers.length === 0, `Cannot handle struct field with type: ${field.type}`);

    return `
      ${scopeCode}
      ${tagCode || retagCode}
      ${fieldWalkName}(traverser, ${fieldCode}, ctx);
    `;
  });

  if (visitedFields.length > 0) fieldsCodes.push('ctx.pop_stack(pop_token);');

  const typeSnakeName = camelToSnake(type.name);
  return `
    pub(crate) unsafe fn walk_${typeSnakeName}<'a, Tr: Traverse<'a>>(
      traverser: &mut Tr,
      node: *mut ${type.rawName},
      ctx: &mut TraverseCtx<'a>
    ) {
      traverser.enter_${typeSnakeName}(&mut *node, ctx);
      ${fieldsCodes.join('\n')}
      ${scopeExitField ? '' : exitScopeCode}
      traverser.exit_${typeSnakeName}(&mut *node, ctx);
    }
  `.replace(/\n\s*\n+/g, '\n');
}

function makeFieldCode(field) {
  return `(node as *mut u8).add(ancestor::${field.offsetVarName}) as *mut ${field.typeName}`;
}

/**
 * @param {EnumType} type
 * @param {Types} types
 */
function generateWalkForEnum(type, types) {
  const variantCodes = type.variants.map((variant) => {
    const variantType = types[variant.innerTypeName];
    assert(variantType, `Cannot handle enum variant with type: ${variant.type}`);

    let nodeCode = 'node';
    if (variant.wrappers.length === 1 && variant.wrappers[0] === 'Box') {
      nodeCode = '(&mut **node)';
    } else {
      assert(variant.wrappers.length === 0, `Cannot handle enum variant with type: ${variant.type}`);
    }

    return `${type.name}::${variant.name}(node) => ` +
      `walk_${camelToSnake(variant.innerTypeName)}(traverser, ${nodeCode} as *mut _, ctx),`;
  });

  const missingVariants = [];
  for (const inheritedTypeName of type.inherits) {
    // Recurse into nested inherited types
    const variantMatches = [],
      inheritedFrom = [inheritedTypeName];
    for (let i = 0; i < inheritedFrom.length; i++) {
      const inheritedTypeName = inheritedFrom[i],
        inheritedType = types[inheritedTypeName];
      if (!inheritedType || inheritedType.kind !== 'enum') {
        missingVariants.push(inheritedTypeName);
      } else {
        variantMatches.push(...inheritedType.variants.map(
          variant => `${type.name}::${variant.name}(_)`,
        ));
        inheritedFrom.push(...inheritedType.inherits);
      }
    }

    variantCodes.push(
      `${variantMatches.join(' | ')} => ` +
        `walk_${camelToSnake(inheritedTypeName)}(traverser, node as *mut _, ctx),`,
    );
  }

  assert(missingVariants.length === 0, `Missing enum variants: ${missingVariants.join(', ')}`);

  const typeSnakeName = camelToSnake(type.name);
  return `
    pub(crate) unsafe fn walk_${typeSnakeName}<'a, Tr: Traverse<'a>>(
      traverser: &mut Tr,
      node: *mut ${type.rawName},
      ctx: &mut TraverseCtx<'a>
    ) {
      traverser.enter_${typeSnakeName}(&mut *node, ctx);
      match &mut *node {
        ${variantCodes.join('\n')}
      }
      traverser.exit_${typeSnakeName}(&mut *node, ctx);
    }
  `;
}
