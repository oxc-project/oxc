import {getSchema} from '../index.js';

const defs = JSON.parse(getSchema());

const typesById = [],
    typesByName = new Map(),
    kindClasses = new Map();

export function registerKindClass(kind, Klass) {
    kindClasses.set(kind, Klass);
}

export function init() {
    // Init types
    const root = getTypeById(0);

    // Calculate niches
    for (const type of typesByName.values()) {
        type.getNiche();
    }

    return root;
}

export function getTypeById(id) {
    let type = typesById[id];
    if (!type) {
        const def = defs[id],
            Klass = kindClasses.get(def.kind);
        type = new Klass();
        typesById[id] = type;
        type.initFromDef(def);
        typesByName.set(type.name, type);
    }
    return type;
}

export function getTypeByName(name) {
    return typesByName.get(name);
}

export function createType(name, type) {
    return typesByName.set(name, type);
}

export class Kind {
    name = null;
    serName = null;
    size = null;
    align = null;
    niche = null;
    deserializerCode = null;
    
    initFromDef(def) {
        this.name = def.name.replace(/<(.)/g, (_, c) => c.toUpperCase())
            .replace(/[>, ]/g, '')
            .replace(/^&(.)/, (_, c) => `Ref${c.toUpperCase()}`)
            .replace(/^(.)/, (_, c) => c.toUpperCase());
        this.serName = def.serName;
        this.size = def.size;
        this.align = def.align;
    }

    getNiche() {
        if (this.niche === null) this.niche = this.calculateNiche();
        return this.niche;
    }

    deserializerName() {
        return `deserialize${this.name}`;
    }

    generateDeserializer(deser) {
        if (this.deserializerCode !== null) return;

        this.deserializerCode = '';
        deser.output(this);
        this.deserializerCode = `function ${this.deserializerName()}(pos) {
            ${this.generateDeserializerBody(deser)}
        }`;
    }

    generateDeserializerBody(_deser) {
        throw new Error(`No generateDeserializerBody implementation for ${this.name}`);
    }

    generateDeserializerCall(posStr, deser) {
        this.generateDeserializer(deser);
        return `${this.deserializerName()}(${posStr})`;
    }
}

export class Niche {
    offset = null;
    size = null;
    min = null;
    max = null;

    constructor({offset, size, min, max}) {
        this.offset = offset;
        this.size = size;
        this.min = min;
        this.max = max;
    }

    static empty() {
        return new Niche({offset: 0, size: 0, min: 0, max: 0});
    }

    isEmpty() {
        return this.size === 0;
    }

    numValues() {
        if (this.isEmpty()) return 0;
        return this.max - this.min + 1;
    }

    clone() {
        return new Niche(this);
    }

    consume() {
        if (this.max === this.min) return Niche.empty();
        return new Niche({offset: this.offset, size: this.size, min: this.min + 1, max: this.max});
    }
}

export function posWithOffsetAndShift(offset, shift) {
    return shift
        ? offset ? `(pos + ${offset}) >> ${shift}` : `pos >> ${shift}`
        : offset ? `pos + ${offset}` : 'pos';
}

export function getFunctionBody(fn) {
    let code = fn.toString();
    return code.slice(code.indexOf('{') + 1, code.lastIndexOf('}'));
}
