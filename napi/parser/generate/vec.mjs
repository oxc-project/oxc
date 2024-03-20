import {Kind, Niche, getTypeById, registerKindClass} from './common.mjs';

export class Vec extends Kind {
    type = null;

    initFromDef(def) {
        super.initFromDef(def);
        this.type = getTypeById(def.valueTypeId);
    }

    calculateNiche() {
        return new Niche({offset: 0, size: 8, min: 0, max: 0});
    }

    generateDeserializerBody(deser) {
        return `
            const arr = [],
                pos32 = pos >> 2,
                len = uint32[pos32 + 6];
            pos = uint32[pos32];
            for (let i = 0; i < len; i++) {
                arr.push(${this.type.generateDeserializerCall('pos', deser)});
                pos += ${this.type.size};
            }
            return arr;
        `;
    }
}
registerKindClass('vec', Vec);
