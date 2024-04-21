import {readFile, writeFile} from 'fs/promises';
import {join as pathJoin, dirname} from 'path';
import {fileURLToPath} from 'url';
import assert from 'assert';
import {format} from 'prettier';
import {Kind, Niche, init, getTypeByName, getFunctionBody} from './common.mjs';
import {StructField} from './struct.mjs';
import {Enum} from './enum.mjs';
import './box.mjs';
import './vec.mjs';
import './option.mjs';
import './cell.mjs';
import './strRef.mjs';
import './primitive.mjs';

const __dirname = dirname(fileURLToPath(import.meta.url));

console.log(`Generating deserializer on ${process.platform}-${process.arch}`)

class DeserializerGenerator {
    types = [];

    generate(type) {
        type.generateDeserializer(this);
        return this.types.map(type => type.deserializerCode).join('\n\n');
    }

    output(type) {
        this.types.push(type);
    }
}

const programType = init();

function customDeserializer(typeName, deserialize) {
    const type = getTypeByName(typeName);
    type.generateDeserializerCall = Kind.prototype.generateDeserializerCall;
    type.generateDeserializerBody = () => getFunctionBody(deserialize);
}

customDeserializer('RegExpFlags', (pos) => {
    const bits = uint8[pos];
    let text = '';
    if (bits & 1) text += 'g';
    if (bits & 2) text += 'i';
    if (bits & 4) text += 'm';
    if (bits & 8) text += 's';
    if (bits & 16) text += 'u';
    if (bits & 32) text += 'y';
    if (bits & 64) text += 'd';
    if (bits & 128) text += 'v';
    return text;
});

customDeserializer('ReferenceFlag', (pos) => {
    const bits = uint8[pos],
        parts = [];
    if (bits & 1) parts.push('Read');
    if (bits & 2) parts.push('Write');
    if (bits & 4) parts.push('Type');
    return parts.join(' | ');
});

// Combine nested enums
function flattenEnumVariants(destTypeName, srcTypeName, variantName) {
    const destType = getTypeByName(destTypeName),
        variantIndex = destType.variants.findIndex(variant => variant.name === variantName);
    assert(variantIndex !== -1);
    destType.variants.splice(
        variantIndex,
        1,
        ...getTypeByName(srcTypeName).variants.map(variant => variant.clone())
    );
}

flattenEnumVariants('Statement', 'Declaration', 'Declaration');
getTypeByName('Statement').niche = new Niche({offset: 0, size: 1, min: 255, max: 255});
getTypeByName('Declaration').niche = new Niche({offset: 0, size: 1, min: 255, max: 255});

// flattenEnumVariants('ExportDefaultDeclarationKind', 'Expression', 'Expression');
// getTypeByName('Expression').niche = new Niche({offset: 0, size: 1, min: 255, max: 255});

getTypeByName('AssignmentTargetMaybeDefault').variants[1].discriminant = 0;
flattenEnumVariants('AssignmentTargetMaybeDefault', 'AssignmentTarget', 'AssignmentTarget');
getTypeByName('AssignmentTargetMaybeDefault').niche = new Niche({offset: 0, size: 1, min: 255, max: 255});
getTypeByName('AssignmentTarget').niche = new Niche({offset: 0, size: 1, min: 255, max: 255});
getTypeByName('AssignmentTargetRest').niche = new Niche({offset: 0, size: 1, min: 0, max: 0});

function combineRest(typeName, arrayFieldName) {
    const type = getTypeByName(typeName),
        restField = type.getFieldByName('rest'),
        arrField = type.getFieldByName(arrayFieldName);
    restField.skip = true;

    const {generatePreamble} = type;
    type.generatePreamble = function(deser) {
        return `
            ${generatePreamble.call(this, deser)}
            const ${arrayFieldName} = ${StructField.prototype.generateDeserializer.call(arrField, deser)},
                rest = ${restField.generateDeserializer(deser)};
            if (rest) ${arrayFieldName}.push(rest);
        `;
    };

    arrField.generateDeserializer = () => arrayFieldName;
}

combineRest('ArrayAssignmentTarget', 'elements');
combineRest('ArrayPattern', 'elements');
combineRest('ObjectAssignmentTarget', 'properties');
combineRest('ObjectPattern', 'properties');
combineRest('FormalParameters', 'items');

const formalParametersRestField = getTypeByName('FormalParameters').getFieldByName('rest'),
    oldRestType = formalParametersRestField.type,
    formalParameterRest = oldRestType.clone('FormalParameterRest');
formalParametersRestField.type = formalParameterRest;
formalParameterRest.generateDeserializerBody = deser => (`
    let rest = ${oldRestType.generateDeserializerCall('pos', deser)};
    if (!rest) return null;
    const {typeAnnotation, optional, ...argument} = rest.argument;
    rest.argument = argument;
    rest.typeAnnotation = typeAnnotation;
    rest.optional = optional;
    return rest;
`);

getTypeByName('ArrayExpressionElement').getVariantByName('Elision').generateDeserializer = () => 'null';

// Alter `BindingPattern` to handle flattened `kind` field without `...`
const bindingPattern = getTypeByName('BindingPattern'),
    kindFieldIndex = bindingPattern.fields.findIndex(field => field.name === 'kind'),
    kindField = bindingPattern.fields[kindFieldIndex],
    otherFields = bindingPattern.fields.filter(field => field !== kindField);
assert(kindField.offset === 0, "BindingPattern's kind field does not have 0 offset");
bindingPattern.variants = kindField.type.variants.map((variant) => {
    variant = variant.clone();
    let variantType = variant.type.type; // Extra `.type` to get type inside box
    if (variant.name === 'BindingIdentifier') {
        variantType = variantType.clone('BindingPatternIdentifier');
        variantType.serName = 'Identifier';
    }
    variant.type = variantType;

    const fieldsBefore = [],
        fieldsAfter = [];
    for (let [index, field] of otherFields.entries()) {
        field = field.clone();
        field.offset -= 8;
        field.oldGenerateDeserializer = field.generateDeserializer;
        field.generateDeserializer = () => field.key;
        (index < kindFieldIndex ? fieldsBefore : fieldsAfter).push(field);
    }
    variantType.fields = [...fieldsBefore, ...variantType.fields, ...fieldsAfter];
    const {generatePreamble} = variantType;
    variantType.generatePreamble = (deser) => {
        const preambles = fieldsBefore.concat(fieldsAfter).map(
            field => `${field.key} = ${field.oldGenerateDeserializer(deser)}`
        );
        return `const ${preambles.join(', ')}; pos = uint32[pos >> 2];`
            + generatePreamble.call(variantType, deser).trimStart();
    };

    return variant;
});
delete bindingPattern.tag;
delete bindingPattern.fields;
delete bindingPattern.transparent;
Object.setPrototypeOf(bindingPattern, Enum.prototype);

// TODO: Why is niche incorrectly identified?
// Looks like the rule is not necessarily to choose the niche with most possibilities.
const bindingIdentifier = getTypeByName('BindingIdentifier'),
    bindingIdentifierNameField = bindingIdentifier.getFieldByName('name');
bindingIdentifier.niche = bindingIdentifierNameField.type.niche.clone();
bindingIdentifier.niche.offset += bindingIdentifierNameField.offset;

const deser = new DeserializerGenerator();

let code = '// Code generated by `generate/index.mjs`. Do not edit.\n\n'
    + await readFile(pathJoin(__dirname, 'base.js'), 'utf8')
    + '\n'
    + deser.generate(programType)
    + '\n';
code = await format(code, {filepath: '.js', tabWidth: 4, singleQuote: true, printWidth: 100});
await writeFile(pathJoin(__dirname, '..', 'deserialize.js'), code);
