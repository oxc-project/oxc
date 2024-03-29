import {Kind, Niche, registerKindClass} from './common.mjs';

export class Primitive extends Kind {
    initFromDef(def) {
        super.initFromDef(def);
    }

    calculateNiche() {
        if (this.name === 'Bool') return new Niche({offset: 0, size: 1, min: 2, max: 255});
        if (this.name.startsWith('NonZero')) return new Niche({offset: 0, size: this.size, min: 0, max: 0});
        return Niche.empty();
    }

    generateDeserializerCall(posStr, _deser) {
        return deserializerCallGenerators[this.name](posStr);
    }
}
registerKindClass('primitive', Primitive);

const deserializerCallGenerators = {
    U8: posStr => `uint8[${posStr}]`,
    U32: posStr => `uint32[(${posStr}) >> 2]`,
    F64: posStr => `float64[(${posStr}) >> 3]`,
    Bool: posStr => `uint8[${posStr}] === 1`,
};
