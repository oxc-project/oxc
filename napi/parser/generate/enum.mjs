import {Kind, Niche, getTypeById, registerKindClass, posWithOffsetAndShift} from './common.mjs';

export class Enum extends Kind {
    variants;

    initFromDef(def) {
        super.initFromDef(def);
        this.variants = def.variants.map(variantDef => EnumVariant.fromDef(variantDef));
    }

    calculateNiche() {
        if (this.variants.length === 0) return Niche.empty();
        // TODO: Handle single-variant enums
        // TODO: Handle enums where one of variants is same size as Enum i.e. discriminant is niched
        // TODO: Handle nested enums

        let minDiscrim = Infinity, maxDiscrim = 0;
        for (const {discriminant} of this.variants) {
            if (discriminant < minDiscrim) minDiscrim = discriminant;
            if (discriminant > maxDiscrim) maxDiscrim = discriminant;
        }

        if (minDiscrim > 0) return new Niche({offset: 0, size: 1, min: 0, max: minDiscrim - 1});
        if (maxDiscrim < 255) return new Niche({offset: 0, size: 1, min: maxDiscrim + 1, max: 255});
        return Niche.empty();
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
