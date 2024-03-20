import {Kind, Niche, registerKindClass, getFunctionBody} from './common.mjs';

export class RefStr extends Kind {
    initFromDef(def) {
        super.initFromDef(def);
    }

    calculateNiche() {
        return new Niche({offset: 0, size: 8, min: 0, max: 0});
    }

    generateDeserializerBody(_deser) {
        return getFunctionBody(deserializeRefStr);
    }
}
registerKindClass('strSlice', RefStr);

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
