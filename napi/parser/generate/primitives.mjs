const deserializerCallGenerators = {
    U8: posStr => `uint8[${posStr}]`,
    U32: posStr => `uint32[(${posStr}) >> 2]`,
    F64: posStr => `float64[(${posStr}) >> 3]`,
    Bool: posStr => `uint8[${posStr}] === 1`,
};

export default function generatePrimitiveDeserializerCallGenerator(type) {
    const generator = deserializerCallGenerators[type.name];
    if (generator) type.generateDeserializerCall = generator;
}
