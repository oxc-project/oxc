import {readFile, writeFile} from 'fs/promises';
import {join as pathJoin, dirname} from 'path';
import {fileURLToPath} from 'url';
import assert from 'assert';
import {format} from 'prettier';
import {getSchema} from '../index.js';

import generatePrimitiveDeserializer from './primitives.mjs';
import {
    generateStructDeserializer,
    generateEnumDeserializer,
    generateBoxDeserializer,
    generateVecDeserializer,
    generateOptionDeserializer
} from './structs.mjs';
import {customTypeNames, generateCustomDeserializer} from './custom.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));

console.log(`Generating deserializer on ${process.platform}-${process.arch}`)

// Get schema
let types = JSON.parse(getSchema());

// Conform type names and flatten cells
let typesByName = Object.create(null);
for (const [index, type] of types.entries()) {
    // Flatten cells + transparent structs
    if (type.kind === 'cell') {
        types[index] = types[type.valueTypeId];
        continue;
    }
    if (type.kind === 'struct' && type.transparent && type.fields[0].offset === 0) {
        types[index] = types[type.fields[0].typeId];
        continue;
    }

    // Conform type name
    type.name = type.name
        .replace(/<(.)/g, (_, c) => c.toUpperCase())
        .replace(/[>, ]/g, '')
        .replace(/^&(.)/, (_, c) => `Ref${c.toUpperCase()}`)
        .replace(/^(.)/, (_, c) => c.toUpperCase());
    assert(!typesByName[type.name], `Repeated type name ${type.name}`);
    typesByName[type.name] = type;

    type.dependencies = new Set();
    type.niche = null;
    type.deserializerName = `deserialize${type.name}`;
    type.isOutput = false;
}

// Prep for combining `rest` fields
for (const typeName of [
    'ArrayPattern', 'ObjectPattern',
    'ArrayAssignmentTarget', 'ObjectAssignmentTarget',
    'FormalParameters'
]) {
    typesByName[typeName].fields.find(field => field.name === 'rest').skip = false;
}

// Combine nested enums
function flattenEnumVariants(destTypeName, srcTypeName, variantName) {
    const destType = typesByName[destTypeName],
        variantIndex = destType.variants.findIndex(variant => variant.name === variantName);
    assert(variantIndex !== -1);
    destType.variants.splice(
        variantIndex,
        1,
        ...typesByName[srcTypeName].variants.map(variant => ({...variant}))
    );
}

flattenEnumVariants('Statement', 'Declaration', 'Declaration');
typesByName.Declaration.niche = {offset: 0, size: 1, min: 255, max: 255};

// flattenEnumVariants('ExportDefaultDeclarationKind', 'Expression', 'Expression');
// typesByName.Expression.niche = {offset: 0, size: 1, min: 255, max: 255};

typesByName.AssignmentTargetMaybeDefault.variants[1].discriminant = 0;
flattenEnumVariants('AssignmentTargetMaybeDefault', 'AssignmentTarget', 'AssignmentTarget');
typesByName.AssignmentTargetMaybeDefault.niche = {offset: 0, size: 1, min: 255, max: 255};

typesByName.Atom.niche = {offset: 0, size: 8, min: 0, max: 0};
typesByName.RegExpFlags.niche = false;
typesByName.ReferenceFlag.niche = false;

// Link up types.
// Delete skipped fields from structs.
const structs = [];
for (const type of Object.values(typesByName)) {
    if (type.kind === 'struct') {
        // Remove skipped fields, and get types for fields
        type.fields = type.fields.flatMap((field) => {
            const {serName, typeId, skip, name: _name, ...fieldProps} = field;
            if (skip) return [];
            return {name: serName, type: types[typeId], ...fieldProps};
        });
        structs.push(type);
    } else if (type.kind === 'enum') {
        // Get types for variants
        let minDiscriminant = Infinity, maxDiscriminant = 0, numTypedVariants = 0;
        type.variants = type.variants.map((variant) => {
            const {valueTypeId, discriminant, ...variantProps} = variant;
            if (discriminant < minDiscriminant) minDiscriminant = discriminant;
            if (discriminant > maxDiscriminant) maxDiscriminant = discriminant;

            let variantType = null;
            if (valueTypeId !== null) {
                variantType = types[valueTypeId];
                type.dependencies.add(variantType);
                numTypedVariants++;
            }
            return {discriminant, type: variantType, ...variantProps};
        });

        // Calculate niche
        if (type.niche === null) {
            if (minDiscriminant === 0) {
                if (maxDiscriminant === 255) {
                    type.niche = false;
                } else {
                    type.niche = {
                        offset: 0,
                        size: 1,
                        min: maxDiscriminant + 1,
                        max: 255,
                    };
                }
            } else {
                type.niche = {
                    offset: 0,
                    size: 1,
                    min: 0,
                    max: minDiscriminant - 1,
                };
            }
        }

        // Check either all variants are typed, or none are
        if (numTypedVariants === 0) {
            type.isTyped = false;
        } else {
            assert(numTypedVariants === type.variants.length);
            type.isTyped = true;
        }
    } else if (type.kind === 'vec' || type.kind === 'box' || type.kind === 'option') {
        const childType = types[type.valueTypeId];
        delete type.valueTypeId;
        type.type = childType;
        type.dependencies.add(childType);

        if (type.kind !== 'option') {
            type.niche = {offset: 0, size: 8, min: 0, max: 0};
        } else if (type.size > childType.size) {
            type.niche = {offset: 0, size: 1, min: 2, max: 255};
        }
    } else if (type.kind === 'primitive') {
        if (type.name === 'Bool') {
            type.niche = {offset: 0, size: 1, min: 2, max: 255};
        } else if (type.name.startsWith('NonZero')) {
            type.niche = {offset: 0, size: type.size, min: 0, max: 0};
        } else {
            type.niche = false;
        }
    } else {
        assert(type.kind === 'strSlice', `Unexpected type kind '${type.kind}'`);
        type.niche = {offset: 0, size: 8, min: 0, max: 0};
    }
}

// Flatten struct fields tagged with `serde(flatten)` + get dependencies for structs
for (const type of structs) {
    const {fields} = type;
    for (let i = 0; i < fields.length; i++) {
        const field = fields[i];
        if (field.flatten && field.type.kind === 'struct') {
            fields.splice(
                i, 1,
                ...field.type.fields.map(child => ({...child, offset: field.offset + child.offset}))
            );
            // Go over these fields again, in case they're recursively flattened
            i--;
            continue;
        }

        type.dependencies.add(field.type);
    }
}

// Calculate niches for structs + options
const withNoNiche = new Set();
for (const type of Object.values(typesByName)) {
    if (type.niche === null) withNoNiche.add(type);
}

while (withNoNiche.size > 0) {
    let someNichesResolved = false;
    outer: for (const type of withNoNiche) {
        if (type.kind === 'option') {
            const childNiche = type.type.niche;
            if (childNiche === null) continue;

            if (childNiche === false) {
                type.niche = false;
            } else if (childNiche.max - childNiche.min === 0) {
                // Only 1 niche in child and `None` will take it
                type.niche = false;
            } else {
                type.niche = {...childNiche, min: childNiche.min + 1};
            }
        } else {
            assert(type.kind === 'struct', `${type.name} has no niche defined`);

            let fieldWithNiche,
                numNiches = 0;
            for (const field of type.fields) {
                const fieldNiche = field.type.niche;
                if (fieldNiche === null) continue outer;
                if (!fieldNiche) continue;

                // Choose field with largest number of niches
                const fieldNumNiches = fieldNiche.max - fieldNiche.min + 1;
                if (fieldNumNiches > numNiches) {
                    fieldWithNiche = field;
                    numNiches = fieldNumNiches;
                }
            }

            if (!fieldWithNiche) {
                type.niche = false;
            } else {
                const fieldNiche = fieldWithNiche.type.niche;
                type.niche = {...fieldNiche, offset: fieldNiche.offset + fieldWithNiche.offset};
            }
        }

        withNoNiche.delete(type);
        someNichesResolved = true;
    }

    if (!someNichesResolved) {
        console.log('Cannot resolve niches for:', [...withNoNiche].map(type => type.name));
        break;
    }
}

// Customize `FormalParameters.rest` field
{
    const type = typesByName.FormalParameters,
        restField = type.fields.find(field => field.name === 'rest');
    type.dependencies.delete(restField.type);
    const restType = {
        ...restField.type,
        dependencies: new Set(restField.type.dependencies),
        name: 'FormalParameterRest',
        deserializerName: 'deserializeFormalParameterRest',
    };
    typesByName.FormalParameterRest = restType;
    restField.type = restType;
    type.dependencies.add(restType);
}

// Set custom types
for (const typeName of customTypeNames) {
    typesByName[typeName].kind = 'custom';
}

// Customize structs/enums
typesByName.ArrayExpressionElement.variants.find(variant => variant.name === 'Elision').code = 'null';

// Generate deserializer
let code = '// Code generated by `generate/index.mjs`. Do not edit.\n\n'
    + await readFile(pathJoin(__dirname, 'base.js')) + '\n';

const generators = {
    primitive: generatePrimitiveDeserializer,
    struct: generateStructDeserializer,
    enum: generateEnumDeserializer,
    box: generateBoxDeserializer,
    vec: generateVecDeserializer,
    option: generateOptionDeserializer,
    custom: generateCustomDeserializer,
};

function generateDeserializer(type) {
    if (type.isOutput) return;
    type.isOutput = true;

    code += generators[type.kind](type) + '\n\n';

    for (const childType of type.dependencies) {
        generateDeserializer(childType);
    }
}
generateDeserializer(typesByName.Program);

code = await format(code, {filepath: '.js', tabWidth: 4, singleQuote: true, printWidth: 100});

await writeFile(pathJoin(__dirname, '../deserialize.js'), code);
