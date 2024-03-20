'use strict';

module.exports = deserialize;

let uint8, uint32, float64, ptrOffset, ptrHigh, endLow, sourceHigh, sourceLow, source;

const textDecoder = new TextDecoder('utf-8', {ignoreBOM: true}),
    decodeStr = textDecoder.decode.bind(textDecoder),
    {fromCodePoint} = String;

function deserialize(buff, sourceBuff) {
    const arrayBuffer = buff.buffer;
    let pos = buff.byteOffset;
    uint8 = pos > 0 ? new Uint8Array(arrayBuffer) : buff;

    uint32 = new Uint32Array(arrayBuffer);
    float64 = new Float64Array(arrayBuffer, 0, arrayBuffer.byteLength >>> 3);

    const pos32 = pos >> 2;
    ptrOffset = uint32[pos32 + 2];
    ptrHigh = uint32[pos32 + 3];
    endLow = uint32[pos32 + 4];
    sourceLow = uint32[pos32 + 6];
    sourceHigh = uint32[pos32 + 7];

    source = sourceBuff;

    const program = deserializeProgram(uint32[pos32]);

    uint8 = uint32 = float64 = undefined;

    return program;
}
