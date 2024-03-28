import {Kind, getTypeById, registerKindClass} from './common.mjs';

export class Cell extends Kind {
    type = null;

    initFromDef(def) {
        super.initFromDef(def);
        this.type = getTypeById(def.valueTypeId);
        this.niche = this.type.niche;
    }

    generateDeserializerCall(posStr, deser) {
        return this.type.generateDeserializerCall(posStr, deser);
    }
}
registerKindClass('cell', Cell);
