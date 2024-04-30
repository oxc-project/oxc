import {spawn} from 'child_process';
import assert from 'assert';

function getDefs() {
    return new Promise((resolve, reject) => {
        const child = spawn('cargo', ['run', '-p', 'oxc_inspect_ast'], {
            stdio: ['pipe', 'pipe', 'inherit']
        });

        let stdout = '';
        child.stdout.on('data', (data) => {
            stdout += data.toString();
        });

        child.on('error', reject);
        child.on('close', (code) => {
            if (code !== 0) {
                reject(new Error(`oxc_inspect_ast exited with code ${code}`));
            } else {
                resolve(stdout);
            }
        });
    });
}

async function getTypes() {
    const defs = JSON.parse(await getDefs());

    for (const type of defs) {
        type.dependencies = [];
        type.dependents = [];
    }

    function addDependency(type, dependency) {
        if (type.dependencies.includes(dependency)) return;
        type.dependencies.push(dependency);
        dependency.dependents.push(type);
    }

    const types = Object.create(null);
    for (const type of defs) {
        types[type.name] = type;
        delete type.serName;

        const {kind} = type;
        if (kind === 'struct') {
            type.fields = type.fields.map((field) => {
                const childType = defs[field.typeId];
                addDependency(type, childType);
                return {name: field.name, type: childType, offset: field.offset};
            });
            delete type.tag;
            delete type.transparent;
        } else if (kind === 'enum') {
            type.variants = type.variants.filter(variant => variant.name !== 'Dummy');
            type.variants = type.variants.map((variant) => {
                const childType = variant.valueTypeId === null ? null : defs[variant.valueTypeId];
                if (childType) addDependency(type, childType);
                return {name: variant.name, type: childType};
            });
            delete type.tag;
        } else if (['vec', 'box', 'option', 'cell'].includes(kind)) {
            const childType = defs[type.valueTypeId];
            type.value = childType;
            addDependency(type, childType);
            delete type.valueTypeId;
        } else {
            assert(['primitive', 'strSlice'].includes(kind), `Unexpected type kind: ${kind}`);
        }
    }
    return types;
}

const types = await getTypes();
// console.dir(types, {depth: 4});

console.log('--------------------');
console.log('> Structs with excess padding:');
for (const type of Object.values(types)) {
    if (type.kind !== 'struct') continue;

    // Calculate size of struct if packed optimally
    const fields = type.fields.map(field => ({...field}));
    fields.sort((f1, f2) => f1.type.align > f2.type.align ? -1 : 1);
    let optimalSize = fields.reduce((size, field) => size + field.type.size, 0);
    const mod = optimalSize % type.align;
    if (mod !== 0) optimalSize += type.align - mod;

    if (optimalSize < type.size) {
        fields.sort((f1, f2) => f1.offset < f2.offset ? -1 : 1);
        let last_offset = 0;
        const gaps = fields.filter((field) => {
            const hasGapBefore = field.offset > last_offset;
            last_offset = field.offset + field.type.size;
            return hasGapBefore;
        });
        console.log(type.name, gaps.map(field => field.name));
    }
}

console.log('--------------------');
console.log('> Nested enums:');
const nestedEnums = Object.values(types).flatMap((type) => {
    if (type.kind !== 'enum') return [];
    if (type.name === 'FakeForTestingInheritedTypes') return [];
    return type.variants.filter(({type: variantType}) => (
        variantType?.kind === 'enum'
        || (variantType?.kind === 'box' && variantType.value.kind === 'enum')
    )).map(variant => `${type.name} -> ${variant.type.name}`);
}).sort();
console.log(nestedEnums.join('\n'));

console.log('--------------------');
console.log('> Structs that need boxing:')
const structsThatNeedBoxing = Object.values(types).filter(type => (
    type.kind === 'struct'
    && type.dependents.some(
        dep => dep.kind !== 'box' && dep.kind !== 'vec'
            && !(dep.kind === 'option' && dep.dependents.every(depDep => depDep.kind === 'vec'))
    )
    && ![
        'Atom', 'Span', 'SourceType', 'ReferenceId', 'SymbolId', 'EmptyObject', 'RegExp',
        'TemplateElementValue', 'Modifiers', 'JSXOpeningFragment', 'JSXClosingFragment'
    ].includes(type.name)
)).map(type => type.name).sort();
console.log(structsThatNeedBoxing.join('\n'));

// console.log(types.IdentifierName.dependents.map(t => t.name));

/*
console.log('--------------------');
console.log('> Type sizes:');
Object.values(types).filter(({kind}) => kind === 'struct' || kind === 'enum')
    .sort((t1, t2) => {
        if (t1.kind < t2.kind) return -1;
        if (t1.kind > t2.kind) return 1;
        return t1.name < t2.name ? -1 : 1;
    })
    .forEach(type => console.log(`${type.name}: ${type.size}`));
*/
