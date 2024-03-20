'use strict';

module.exports = deserialize;

let uint8, uint32, float64, source, sourceIsAscii, sourceLen;

const textDecoder = new TextDecoder('utf-8', {ignoreBOM: true}),
    decodeStr = textDecoder.decode.bind(textDecoder),
    {fromCodePoint} = String;

function deserialize(buff, sourceStr, sourceByteLen) {
    uint8 = buff;
    uint32 = new Uint32Array(buff.buffer);
    float64 = new Float64Array(buff.buffer);

    source = sourceStr;
    sourceLen = sourceByteLen;
    sourceIsAscii = sourceStr.length === sourceByteLen;

    // (2 * 1024 * 1024 * 1024 - 16) >> 2
    const metadataPos32 = 536870908;
    const program = deserializeProgram(uint32[metadataPos32]);

    uint8 = uint32 = float64 = undefined;

    return program;
}
