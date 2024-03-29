import {Kind, Niche, getTypeById, registerKindClass} from './common.mjs';

export class Box extends Kind {
    type = null;

    initFromDef(def) {
        super.initFromDef(def);
        this.type = getTypeById(def.valueTypeId);
    }

    calculateNiche() {
        return new Niche({offset: 0, size: 8, min: 0, max: 0});
    }

    generateDeserializerBody(deser) {
        return `return ${this.type.generateDeserializerCall('uint32[pos >> 2]', deser)};`;
    }
}
registerKindClass('box', Box);
