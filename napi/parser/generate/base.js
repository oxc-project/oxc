'use strict';

module.exports = deserialize;

let uint8, uint32, float64,
    ptrMask, buffLow, buffHigh, buffEndLow, sourceLow, sourceHigh,
    source, sourceIsAscii;

const textDecoder = new TextDecoder('utf-8', {ignoreBOM: true}),
    decodeStr = textDecoder.decode.bind(textDecoder),
    {fromCodePoint} = String;

function deserialize(buff, sourceBuff, isAscii) {
    uint8 = buff;
    uint32 = new Uint32Array(buff.buffer, buff.byteOffset, buff.byteLength >>> 2);
    float64 = new Float64Array(buff.buffer, buff.byteOffset, buff.byteLength >>> 3);

    const pos = uint32[0];
    ptrMask = uint32[1];
    buffLow = uint32[2];
    buffHigh = uint32[3];
    buffEndLow = uint32[4];
    sourceLow = uint32[5];
    sourceHigh = uint32[6];

    source = sourceBuff;
    sourceIsAscii = isAscii;

    const program = deserializeProgram(pos);

    uint8 = uint32 = float64 = undefined;

    return program;
}
