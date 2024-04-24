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
                return {name: field.name, type: childType};
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

const structsThatNeedBoxing = Object.values(types).filter(type => (
    type.kind === 'struct'
    && type.dependents.some(dep => dep.kind !== 'box' && dep.kind !== 'vec' && dep.kind !== 'enum'
    && ![
        'Atom', 'Span', 'SourceType', 'ReferenceId', 'SymbolId', 'EmptyObject', 'RegExp',
        'TemplateElementValue', 'IdentifierName', 'Modifiers', 'JSXOpeningFragment', 'JSXClosingFragment'
    ].includes(type.name))
));
console.log('Structs that need boxing:', structsThatNeedBoxing.map(type => type.name));

const nestedEnums = Object.values(types).flatMap((type) => {
    if (type.kind !== 'enum') return [];
    return type.variants.filter(({type: variantType}) => (
        variantType?.kind === 'enum'
        || (variantType?.kind === 'box' && variantType.value.kind === 'enum')
    )).map(variant => `${type.name} -> ${variant.type.name}`);
}).sort();
console.log('Nested enums:');
console.log(nestedEnums.join('\n'));

// console.log(types.BindingPattern.dependents.map(t => t.name));
