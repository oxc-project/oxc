export function generateStructDeserializer(type) {
    const {fields} = type;
    let retCode;
    if (fields.length === 0) {
        retCode = 'null';
    } else if (type.transparent) {
        retCode = fields[0].code || generateStructFieldCode(fields[0]);
    } else {
        const fieldCodes = fields.map((field) => {
            const fieldCode = field.code || generateStructFieldCode(field);
            return field.flatten
                ? `...${fieldCode}`
                : `${field.name}: ${fieldCode}`;
        });
        if (type.tag) fieldCodes.unshift(`${type.tag}: '${type.serName}'`);
        retCode = `{ ${fieldCodes.join(',')} }`;
    }

    return `function ${type.deserializerName}(pos) {
        ${type.preamble || ''}
        return ${retCode};
    }`;
}

export function generateStructFieldCode(field) {
    return `${field.type.deserializerName}(${posWithOffsetAndShift(field.offset)})`;
}

export function generateEnumDeserializer(type) {
    if (type.isTyped) return generateTypedEnumDeserializer(type);
    return generateUntypedEnumDeserializer(type);
}

function generateTypedEnumDeserializer(type) {
    const variantCodes = type.variants.map((variant) => {
        const variantCode = variant.code || generateEnumVariantCode(variant);
        return `case ${variant.discriminant}: return ${variantCode};`;
    });

    variantCodes.push(`default: throw new Error(\`Unexpected discriminant \${uint8[pos]} for ${type.name}\`);`);

    return `function ${type.deserializerName}(pos) {
        switch (uint8[pos]) {
            ${variantCodes.join('')}
        }
    }`;
}

function generateEnumVariantCode(variant) {
    return `${variant.type.deserializerName}(pos + ${variant.type.align})`;
}

function generateUntypedEnumDeserializer(type) {
    const variantCodes = type.variants.map(
        variant => `case ${variant.discriminant}: return ${JSON.stringify(variant.serValue)};`
    );

    variantCodes.push(`default: throw new Error(\`Unexpected discriminant \${uint8[pos]} for ${type.name}\`);`);

    return `function ${type.deserializerName}(pos) {
        switch (uint8[pos]) {
            ${variantCodes.join('')}
        }
    }`;
}

export function generateBoxDeserializer(type) {
    return `function ${type.deserializerName}(pos) {
        return ${type.type.deserializerName}(uint32[pos >> 2] - ptrOffset);
    }`;
}

export function generateVecDeserializer(type) {
    return `function ${type.deserializerName}(pos) {
        const arr = [],
            len = uint32[(pos + 24) >> 2];
        pos = uint32[pos >> 2] - ptrOffset;
        for (let i = 0; i < len; i++) {
            arr.push(${type.type.deserializerName}(pos));
            pos += ${type.type.size};
        }
        return arr;
    }`;
}

export function generateOptionDeserializer(type) {
    const child = type.type;

    let noneCondition, valueOffset;
    if (child.size === type.size) {
        // Using niche
        // TODO: Make this work
        valueOffset = 0;

        const {niche} = child;
        if (niche) {
            if (niche.size === 1) {
                noneCondition = `uint8[${posWithOffsetAndShift(niche.offset, 0)}] === ${niche.min}`;
            } else if (niche.size === 8) {
                if (niche.min === 0) {
                    noneCondition = `uint32[${posWithOffsetAndShift(niche.offset, 2)}] === 0`
                        + ` && uint32[${posWithOffsetAndShift(niche.offset + 4, 2)}] === 0`;
                } else {
                    // TODO
                    console.log(`Option with niche size 8 and non-zero niche value:`, type.name, child);
                    noneCondition = 'true';
                }
            } else {
                // TODO
                console.log(`Option with niche size ${niche.size}:`, type.name, child);
                noneCondition = 'true';
            }
        } else {
            console.log('Option with no niche:', type.name);
            noneCondition = 'true'; // TODO
        }
    } else {
        // No niche
        noneCondition = `uint8[pos] === 0`; // TODO: Is this always correct?
        valueOffset = child.align;
    }

    return `function ${type.deserializerName}(pos) {
        if (${noneCondition}) return null;
        return ${child.deserializerName}(${posWithOffsetAndShift(valueOffset)});
    }`;
}

function posWithOffsetAndShift(offset, shift) {
    return shift
        ? offset ? `(pos + ${offset}) >> ${shift}` : `pos >> ${shift}`
        : offset ? `pos + ${offset}` : 'pos';
}
