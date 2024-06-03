import assert from 'assert';
import {camelToSnake, snakeToCamel} from './utils.mjs';

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
            clippy::borrow_as_ptr,
            clippy::cast_ptr_alignment
        )]

        use std::cell::Cell;

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

function generateWalkForStruct(type, types) {
    let scopeIdField;
    const visitedFields = type.fields.filter(field => {
        if (field.name === 'scope_id' && field.typeName === `Cell<Option<ScopeId>>`) {
            scopeIdField = field;
        }
        return field.innerTypeName in types;
    });

    const {scopeArgs} = type;
    let scopeEnterField, enterScopeCode = '', exitScopeCode = '';
    if (scopeArgs && scopeIdField) {
        // Get field to enter scope before
        const enterFieldName = scopeArgs.enter_scope_before;
        if (enterFieldName) {
            scopeEnterField = visitedFields.find(field => field.name === enterFieldName);
            assert(
                scopeEnterField,
                `\`visited_node\` attr says to enter scope before field '${enterFieldName}' `
                + `in '${type.name}', but that field is not visited`
            );
            if (scopeEnterField === visitedFields[0]) scopeEnterField = undefined;
        }

        // TODO: Maybe this isn't quite right. `scope_id` fields are `Cell<Option<ScopeId>>`,
        // so visitor is able to alter the `scope_id` of a node higher up the tree,
        // but we don't take that into account.
        // Visitor should not do that though, so maybe it's OK.
        // In final version, we should not make `scope_id` fields `Cell`s to prevent this.
        enterScopeCode = `
            let mut previous_scope_id = None;
            if let Some(scope_id) = (*(${makeFieldCode(scopeIdField)})).get() {
                previous_scope_id = Some(ctx.current_scope_id());
                ctx.set_current_scope_id(scope_id);
            }
        `;

        exitScopeCode = `
            if let Some(previous_scope_id) = previous_scope_id {
                ctx.set_current_scope_id(previous_scope_id);
            }
        `;
    }

    const fieldsCodes = visitedFields.map((field, index) => {
        const fieldWalkName = `walk_${camelToSnake(field.innerTypeName)}`;

        const retagCode = index === 0
            ? ''
            : `ctx.retag_stack(AncestorType::${type.name}${snakeToCamel(field.name)});`;
        const fieldCode = makeFieldCode(field);
        let scopeCode = '';
        if (field === scopeEnterField) {
            scopeCode = enterScopeCode;
            enterScopeCode = '';
        }

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
                walkVecCode = `walk_statements(traverser, ${fieldCode}, ctx);`
            } else {
                let walkCode = `${fieldWalkName}(traverser, item as *mut _, ctx);`,
                    iterModifier = '';
                if (field.wrappers.length === 2 && field.wrappers[1] === 'Option') {
                    iterModifier = '.flatten()';
                } else {
                    assert(
                        field.wrappers.length === 1,
                        `Cannot handle struct field with type ${field.type}`
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
                ${retagCode}
                ${walkVecCode}
            `;
        }

        if (field.wrappers.length === 1 && field.wrappers[0] === 'Box') {
            return `
                ${scopeCode}
                ${retagCode}
                ${fieldWalkName}(traverser, (&mut **(${fieldCode})) as *mut _, ctx);
            `;
        }

        assert(field.wrappers.length === 0, `Cannot handle struct field with type: ${field.type}`);

        return `
            ${scopeCode}
            ${retagCode}
            ${fieldWalkName}(traverser, ${fieldCode}, ctx);
        `;
    });

    if (visitedFields.length > 0) {
        const field = visitedFields[0],
            fieldCamelName = snakeToCamel(field.name);
        fieldsCodes.unshift(`
            ctx.push_stack(
                Ancestor::${type.name}${fieldCamelName}(
                    ancestor::${type.name}Without${fieldCamelName}(node)
                )
            );
        `);
        fieldsCodes.push('ctx.pop_stack();');
    }

    const typeSnakeName = camelToSnake(type.name);
    return `
        pub(crate) unsafe fn walk_${typeSnakeName}<'a, Tr: Traverse<'a>>(
            traverser: &mut Tr,
            node: *mut ${type.rawName},
            ctx: &mut TraverseCtx<'a>
        ) {
            ${enterScopeCode}
            traverser.enter_${typeSnakeName}(&mut *node, ctx);
            ${fieldsCodes.join('\n')}
            ${enterScopeCode ? '' : exitScopeCode}
            traverser.exit_${typeSnakeName}(&mut *node, ctx);
            ${enterScopeCode ? exitScopeCode : ''}
        }
    `.replace(/\n\s*\n+/g, '\n');
}

function makeFieldCode(field) {
    return `(node as *mut u8).add(ancestor::${field.offsetVarName}) as *mut ${field.typeName}`;
}

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

        return `${type.name}::${variant.name}(node) => `
            + `walk_${camelToSnake(variant.innerTypeName)}(traverser, ${nodeCode} as *mut _, ctx),`;
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
                    variant => `${type.name}::${variant.name}(_)`
                ));
                inheritedFrom.push(...inheritedType.inherits);
            }
        }

        variantCodes.push(
            `${variantMatches.join(' | ')} => `
            + `walk_${camelToSnake(inheritedTypeName)}(traverser, node as *mut _, ctx),`
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
