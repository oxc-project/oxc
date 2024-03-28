import {Kind, Niche, getTypeById, registerKindClass, posWithOffsetAndShift, createType} from './common.mjs';

export class Option extends Kind {
    type = null;

    initFromDef(def) {
        super.initFromDef(def);
        this.type = getTypeById(def.valueTypeId);

        if (this.size > this.type.size) {
            this.niche = new Niche({offset: 0, size: 1, min: 2, max: 255});
        } else if (this.type.niche) {
            this.niche = this.type.niche.consume();
        }
    }

    generateDeserializerBody(deser) {
        const child = this.type;
        let noneCondition, valueOffset;
        if (child.size === this.size) {
            // Using niche
            // TODO: Make this work
            valueOffset = 0;

            const {niche} = child;
            if (niche) {
                if (niche.size === 1) {
                    noneCondition = `uint8[${posWithOffsetAndShift(niche.offset, 0)}] === ${niche.min}`;
                } else if (niche.size === 8) {
                    if (niche.min === 0) {
                        noneCondition = `uint32[${posWithOffsetAndShift(niche.offset, 2)}] === 0`
                            + ` && uint32[${posWithOffsetAndShift(niche.offset + 4, 2)}] === 0`;
                    } else {
                        // TODO
                        console.log(`Option with niche size 8 and non-zero niche value:`, this.name, child);
                        noneCondition = 'true';
                    }
                } else {
                    // TODO
                    console.log(`Option with niche size ${niche.size}:`, this.name, child);
                    noneCondition = 'true';
                }
            } else {
                console.log('Option with no niche:', this.name);
                noneCondition = 'true'; // TODO
            }
        } else {
            // No niche
            noneCondition = `uint8[pos] === 0`; // TODO: Is this always correct?
            valueOffset = child.align;
        }

        return `
            if (${noneCondition}) return null;
            return ${child.generateDeserializerCall(posWithOffsetAndShift(valueOffset), deser)};
        `;
    }

    clone(name) {
        const type = new Option();
        Object.assign(type, this, {
            name,
            serName: name,
            niche: this.niche ? this.niche.clone() : null,
        });
        createType(name, type);
        return type;
    }
}
registerKindClass('option', Option);
