import {generateStructDeserializer, generateStructFieldCode} from './structs.mjs';

const customDeserializers = {
    RefStr: deserializeRefStr,
    RegExpFlags: deserializeRegExpFlags,
    ReferenceFlag: deserializeReferenceFlag,
};

const customDeserializerGenerators = {
    ArrayPattern: generateArrayPatternDeserializer,
    ObjectPattern: generateObjectPatternDeserializer,
    ArrayAssignmentTarget: generateArrayPatternDeserializer,
    ObjectAssignmentTarget: generateObjectPatternDeserializer,
    FormalParameters: generateFormalParametersDeserializer,
    FormalParameterRest: generateFormalParameterRestDeserializer
};

for (const [typeName, deserializer] of Object.entries(customDeserializers)) {
    customDeserializerGenerators[typeName] = () => deserializer.toString();
}

export const customTypeNames = Object.keys(customDeserializerGenerators);

export function generateCustomDeserializer(type) {
    return customDeserializerGenerators[type.name](type);
}

function deserializeRefStr(pos) {
    const pos32 = pos >> 2,
        len = uint32[pos32 + 2];
    if (len === 0) return '';

    pos = uint32[pos32];
    if (sourceIsAscii && pos < sourceLen) return source.substr(pos, len);

    // Longer strings use `TextDecoder`
    // TODO: Find best switch-over point
    const end = pos + len;
    if (len > 50) return decodeStr(uint8.subarray(pos, end));

    // Shorter strings decode by hand to avoid native call
    let out = '',
        c;
    do {
        c = uint8[pos++];
        if (c < 0x80) {
            out += fromCodePoint(c);
        } else {
            out += decodeStr(uint8.subarray(pos - 1, end));
            break;
        }
    } while (pos < end);

    return out;
}

function deserializeRegExpFlags(pos) {
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
}

function deserializeReferenceFlag(pos) {
    const bits = uint8[pos],
        parts = [];
    if (bits & 1) parts.push('Read');
    if (bits & 2) parts.push('Write');
    if (bits & 4) parts.push('Type');
    return parts.join(' | ');
}

function generateArrayPatternDeserializer(type) {
    return generateCombinedRestDeserializer(type, 'elements');
}

function generateObjectPatternDeserializer(type) {
    return generateCombinedRestDeserializer(type, 'properties');
}

function generateFormalParametersDeserializer(type) {
    return generateCombinedRestDeserializer(type, 'items');
}

function generateCombinedRestDeserializer(type, arrayFieldName) {
    type = {...type};
    let arrField, restField;
    type.fields = type.fields.filter((field) => {
        if (field.name === 'rest') {
            restField = field;
            return false;
        } else if (field.name === arrayFieldName) {
            arrField = field;
        }
        return true;
    });

    type.preamble = `
        ${type.preamble || ''}
        const ${arrayFieldName} = ${generateStructFieldCode(arrField)},
            rest = ${generateStructFieldCode(restField)};
        if (rest) ${arrayFieldName}.push(rest);
    `;

    arrField.code = arrayFieldName;

    return generateStructDeserializer(type);
}

function generateFormalParameterRestDeserializer(type) {
    return `function deserializeFormalParameterRest(pos) {
        let rest = deserializeOptionBoxBindingRestElement(pos);
        if (!rest) return null;
        const {typeAnnotation, optional, ...argument} = rest.argument;
        rest.argument = argument;
        rest.typeAnnotation = typeAnnotation;
        rest.optional = optional;
        return rest;
    }`;
}
