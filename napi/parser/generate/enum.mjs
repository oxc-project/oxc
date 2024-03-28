import {Kind, Niche, getTypeById, registerKindClass, posWithOffsetAndShift} from './common.mjs';

export class Enum extends Kind {
    variants;

    initFromDef(def) {
        super.initFromDef(def);

        let minDiscriminant = Infinity, maxDiscriminant = 0;
        this.variants = def.variants.map((variantDef) => {
            const variant = EnumVariant.fromDef(variantDef);

            const {discriminant} = variant;
            if (discriminant < minDiscriminant) minDiscriminant = discriminant;
            if (discriminant > maxDiscriminant) maxDiscriminant = discriminant;

            return variant;
        });
        
        // Calculate niche
        if (minDiscriminant > 0) {
            this.niche = new Niche({offset: 0, size: 1, min: 0, max: minDiscriminant - 1});
        } else if (maxDiscriminant < 255) {
            this.niche = new Niche({offset: 0, size: 1, min: maxDiscriminant + 1, max: 255});
        }
    }

    getVariantByName(name) {
        return this.variants.find(variant => variant.name === name);
    }

    generateDeserializerBody(deser) {
        const variantCodes = this.variants.map(
            variant => `case ${variant.discriminant}: return ${variant.generateDeserializer(deser)};`
        );
        variantCodes.push(`default: throw new Error(\`Unexpected discriminant \${uint8[pos]} for ${this.name}\`);`);

        return `switch (uint8[pos]) {
            ${variantCodes.join('')}
        }`;
    }
}
registerKindClass('enum', Enum);

export class EnumVariant {
    name;
    type;
    discriminant;
    serValue;

    constructor(props) {
        this.name = props.name;
        this.type = props.type;
        this.discriminant = props.discriminant;
        this.serValue = props.serValue;
    }

    static fromDef(def) {
        return new EnumVariant({
            name: def.name,
            type: def.valueTypeId != null ? getTypeById(def.valueTypeId) : null,
            discriminant: def.discriminant,
            serValue: def.serValue,
        });
    }

    clone() {
        return new EnumVariant(this);
    }

    generateDeserializer(deser) {
        return this.type
            ? this.type.generateDeserializerCall(posWithOffsetAndShift(this.type.align), deser)
            : JSON.stringify(this.serValue);
    }
}
