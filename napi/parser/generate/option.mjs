import {Kind, Niche, getTypeById, registerKindClass, posWithOffsetAndShift, createType} from './common.mjs';

export class Option extends Kind {
    type = null;

    initFromDef(def) {
        super.initFromDef(def);
        this.type = getTypeById(def.valueTypeId);
    }

    calculateNiche() {
        if (this.size > this.type.size) return new Niche({offset: 0, size: 1, min: 2, max: 255});
        return this.type.getNiche().consume();
    }

    generateDeserializerBody(deser) {
        const child = this.type;
        let noneCondition, valueOffset;
        if (child.size === this.size) {
            // Using niche
            valueOffset = 0;

            const {niche} = child;
            if (niche.isEmpty()) {
                noneCondition = `uint8[pos] === 0`; // TODO: Is this always correct?
                valueOffset = child.align;
            } else if (niche.size === 1) {
                noneCondition = `uint8[${posWithOffsetAndShift(niche.offset, 0)}] === ${niche.min}`;
            } else if (niche.size === 2) {
                noneCondition = `uint32[${posWithOffsetAndShift(niche.offset, 2)}] & 0xFFFF === ${niche.min}`;
            } else if (niche.size === 4) {
                noneCondition = `uint32[${posWithOffsetAndShift(niche.offset, 2)}] === ${niche.min}`;
            } else if (niche.size === 8) {
                const value = BigInt(niche.min),
                    high32 = value / 0x100000000n, // Top 32 bits
                    low32 = value % 0x100000000n; // Bottom 32 bits
                noneCondition = `uint32[${posWithOffsetAndShift(niche.offset, 2)}] === ${low32}`
                    + ` && uint32[${posWithOffsetAndShift(niche.offset + 4, 2)}] === ${high32}`;
            } else {
                throw new Error(`Option with niche size ${niche.size}: ${this.name}`);
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
            niche: this.niche.clone(),
        });
        createType(name, type);
        return type;
    }
}
registerKindClass('option', Option);
