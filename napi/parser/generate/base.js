'use strict';

module.exports = deserialize;

let uint8, uint32, float64, source, sourceIsAscii, sourceOffset, ptrMask;

const textDecoder = new TextDecoder('utf-8', {ignoreBOM: true}),
    decodeStr = textDecoder.decode.bind(textDecoder),
    {fromCodePoint} = String;

function deserialize(buff, sourceBuff, isAscii) {
    uint8 = buff;
    uint32 = new Uint32Array(buff.buffer, buff.byteOffset, buff.byteLength >> 2);
    float64 = new Float64Array(buff.buffer, buff.byteOffset, buff.byteLength >> 3);

    source = sourceBuff;
    sourceIsAscii = isAscii;
    sourceOffset = uint32[1];
    ptrMask = uint32[2];

    const program = deserializeProgram(uint32[0]);

    uint8 = uint32 = float64 = undefined;

    return program;
}
