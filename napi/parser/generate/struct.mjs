import assert from 'assert';
import {Kind, Niche, getTypeById, registerKindClass, posWithOffsetAndShift, createType} from './common.mjs';

export class Struct extends Kind {
    tag;
    fields;
    transparent = false;

    initFromDef(def) {
        super.initFromDef(def);
        this.tag = def.tag;

        this.fields = def.fields.flatMap((fieldDef) => {
            const field = StructField.fromDef(fieldDef);

            if (field.flatten && field.type instanceof Struct) {
                return field.type.fields.map((flatField) => {
                    flatField = flatField.clone();
                    flatField.offset += field.offset;
                    return flatField;
                });
            }
            
            return [field];
        });

        this.transparent = def.transparent;
    }

    calculateNiche() {
        // Choose field with the largest niche
        let nicheField,
            nicheCount = 0;
        for (const field of this.fields) {
            const count = field.type.getNiche().numValues();
            if (count > nicheCount) {
                nicheField = field;
                nicheCount = count;
            }
        }

        if (!nicheField) return Niche.empty();

        const niche = nicheField.type.niche.clone();
        niche.offset += nicheField.offset;
        return niche;
    }

    getFieldByName(name) {
        return this.fields.find(field => field.name === name);
    }

    generateDeserializerBody(deser) {
        let code;
        const {fields} = this;
        if (fields.length === 0) {
            code = 'null';
        } else if (this.transparent) {
            assert(this.fields.length === 1, `Transparent struct with more than 1 field: ${this.name}`);
            assert(
                this.fields[0].offset === 0,
                `Transparent struct with field not at offset 0: ${this.name}`
            );
            code = fields[0].generateDeserializer(deser);
        } else {
            const fieldCodes = fields.flatMap((field) => {
                if (field.skip) return [];
                const fieldValueCode = field.generateDeserializer(deser);
                const fieldCode = field.flatten
                    ? `...${fieldValueCode}`
                    : `${field.key}: ${fieldValueCode}`
                return [fieldCode];
            });
            if (this.tag) {
                fieldCodes.unshift(`${this.tag}: ${JSON.stringify(this.serName)}`);
            }
            code = `{ ${fieldCodes.join(',')} }`;
        }

        return `
            ${this.generatePreamble(deser)}
            return ${code};
        `;
    }

    generateDeserializerCall(posStr, deser) {
        if (this.transparent) return this.fields[0].type.generateDeserializerCall(posStr, deser);
        return super.generateDeserializerCall(posStr, deser);
    }

    generatePreamble(deser) {
        return '';
    }

    clone(name) {
        const type = new Struct();
        Object.assign(type, this, {
            name,
            serName: name,
            niche: this.niche.clone(),
            fields: this.fields.map(field => field.clone()),
        });
        createType(name, type);
        return type;
    }
}
registerKindClass('struct', Struct);

export class StructField {
    name;
    key;
    type;
    offset;
    skip;
    flatten;

    constructor(props) {
        this.name = props.name;
        this.key = props.key;
        this.type = props.type;
        this.offset = props.offset;
        this.skip = props.skip;
        this.flatten = props.flatten;
    }

    static fromDef(def) {
        return new StructField({
            name: def.name,
            key: def.serName,
            type: getTypeById(def.typeId),
            offset: def.offset,
            skip: def.skip,
            flatten: def.flatten,
        });
    }

    clone() {
        return new StructField(this);
    }

    generateDeserializer(deser) {
        return this.type.generateDeserializerCall(posWithOffsetAndShift(this.offset), deser);
    }
}
