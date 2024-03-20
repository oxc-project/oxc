import {Kind, getTypeById, registerKindClass} from './common.mjs';

export class Cell extends Kind {
    type = null;

    initFromDef(def) {
        super.initFromDef(def);
        this.type = getTypeById(def.valueTypeId);
    }

    calculateNiche() {
        return this.type.getNiche();
    }

    generateDeserializerCall(posStr, deser) {
        return this.type.generateDeserializerCall(posStr, deser);
    }
}
registerKindClass('cell', Cell);
