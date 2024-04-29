// Generate `Traverse` trait Rust code by parsing Rust type definitions.
// This is a quick-and-dirty version written in JS for speed of implementation.
// We should do this properly with a Rust build script using `syn` etc.

import {readFile, writeFile} from 'fs/promises';
import {join as pathJoin} from 'path';
import {fileURLToPath} from 'url';
import assert from 'assert';

const types = await getTypesFromCode();
const traitCode = generateTraitCode(types);

const outPath = pathJoin(fileURLToPath(import.meta.url), '../../src/traverse/traverse.rs');
await writeFile(outPath, traitCode);

function generateTraitCode(types) {
    let methods = '';
    const addMethod = code => { methods += `    ${code}\n` };

    for (const type of Object.values(types)) {
        const {name} = type,
            snakeName = snakeCase(name);
        
        let ty = `Traversable${name}`;
        if (type.hasLifetime) ty += "<'a>";
        
        let isLeaf = false;
        if (type.kind === 'struct') {
            ty = `SharedBox<'a, ${ty}>`;
            isLeaf = !type.fields.some((field) => {
                let fieldType = field.type;
                outer: while (true) {
                    for (const wrapper of ['Box', 'Option', 'Vec', 'Cell']) {
                        if (fieldType.startsWith(`${wrapper}<`)) {
                            fieldType = fieldType.slice(wrapper.length + 1, -1);
                            continue outer;
                        }
                    }
                    break;
                }
                return !!types[fieldType];
            });
        }
        
        if (isLeaf) {
            addMethod(`fn visit_${snakeName}(&mut self, node: ${ty}, ctx: &TraverseCtx<'a>, tk: &mut Token) {}`);
        } else {
            addMethod(`fn enter_${snakeName}(&mut self, node: ${ty}, ctx: &TraverseCtx<'a>, tk: &mut Token) {}`);
            addMethod(`fn exit_${snakeName}(&mut self, node: ${ty}, ctx: &TraverseCtx<'a>, tk: &mut Token) {}`);
        }
        methods += '\n';
    }

    const template = `
        use super::{ast::*, cell::Token, SharedBox, TraverseCtx};

        #[allow(unused_variables)]
        pub trait Traverse<'a> {
        [[methods]]
        }
    `.trimStart().split('\n').map(line => line.trim()).join('\n');

    return template.replace('[[methods]]', methods.trimEnd());
}

async function getTypesFromCode() {
    const codeDirPath = pathJoin(fileURLToPath(import.meta.url), '../../src/ast/');
    const filenames = ['js.rs', 'jsx.rs', 'literal.rs', 'ts.rs'];

    // Parse type defs from Rust files
    const types = Object.create(null);
    for (const filename of filenames) {
        const code = await readFile(`${codeDirPath}${filename}`, 'utf8'),
            lines = code.split(/\r?\n/);
        for (let i = 0; i < lines.length; i++) {
            if (lines[i] === '#[ast_node]') {
                let match;
                while (true) {
                    match = lines[++i].match(/^pub (enum|struct) (.+?)(<'a>)? \{/);
                    if (match) break;
                }
                const [, kind, name, lifetimeStr] = match,
                    hasLifetime = !!lifetimeStr;
                const itemLines = [];
                while (true) {
                    const line = lines[++i].replace(/\/\/.*$/, '').replace(/\s+/g, ' ').trim();
                    if (line === '}') break;
                    if (line !== '') itemLines.push(line);
                }

                if (kind === 'enum') {
                    const variants = [],
                        inherits = [];
                    for (const line of itemLines) {
                        const match = line.match(/^(.+?)\((.+?)\)(?: ?= ?(\d+))?,$/);
                        if (match) {
                            let [, name, type, discriminant] = match;
                            type = type.replace(/<'a>/g, '').replace(/<'a,\s*/g, '<');
                            discriminant = discriminant ? +discriminant : null;
                            variants.push({name, type, discriminant});
                        } else {
                            const match2 = line.match(/^@inherit ([A-Za-z]+)$/);
                            assert(match2, `Cannot parse line ${i} in '${filename}' as enum variant: '${line}'`);
                            inherits.push(match2[1]);
                        }
                    }
                    types[name] = {kind: 'enum', name, hasLifetime, variants, inherits};
                } else {
                    const fields = [];
                    for (let i = 0; i < itemLines.length; i++) {
                        const line = itemLines[i];
                        if (line.startsWith('#[')) {
                            while (!itemLines[i].endsWith(']')) {
                                i++;
                            }
                            continue;
                        }

                        const match = line.match(/^pub (?:r#)?([a-z_]+): (.+),(?: ?\/\/.+)?$/);
                        assert(match, `Cannot parse line ${i} in '${filename}' as struct field: '${line}'`);
                        let [, name, type] = match;
                        type = type.replace(/<'a>/g, '').replace(/<'a, ?/g, '<');
                        fields.push({name, type});
                    }
                    types[name] = {kind: 'struct', name, hasLifetime, fields};
                }
            }
        }
    }
    return types;
}

function snakeCase(name) {
    let prefixLen = 1;
    for (const prefix of ['TS', 'JSX', 'JS']) {
        if (name.startsWith(prefix)) {
            prefixLen = prefix.length;
            break;
        }
    }
    return name.slice(0, prefixLen).toLowerCase()
        + name.slice(prefixLen).replace(/[A-Z]/g, c => `_${c.toLowerCase()}`);
}
