const deserializers = {
    U8: deserializeU8,
    U32: deserializeU32,
    F64: deserializeF64,
    Bool: deserializeBool
};

export default function generatePrimitiveDeserializer(type) {
    return deserializers[type.name].toString();
}

function deserializeU8(pos) {
    return uint8[pos];
}

function deserializeU32(pos) {
    return uint32[pos >> 2];
}

function deserializeBool(pos) {
    return uint8[pos] === 1;
}

function deserializeF64(pos) {
    return float64[pos >> 3];
}
