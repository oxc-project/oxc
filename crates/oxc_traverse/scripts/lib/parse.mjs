import {readFile} from 'fs/promises';
import {join as pathJoin} from 'path';
import {fileURLToPath} from 'url';
import assert from 'assert';
import {typeAndWrappers} from './utils.mjs';

const FILENAMES = ['js.rs', 'jsx.rs', 'literal.rs', 'ts.rs'];

/**
 * Parse type defs from Rust files.
 */
export default async function getTypesFromCode() {
    const codeDirPath = pathJoin(fileURLToPath(import.meta.url), '../../../../oxc_ast/src/ast/');

    const types = Object.create(null);
    for (const filename of FILENAMES) {
        const code = await readFile(`${codeDirPath}${filename}`, 'utf8');
        parseFile(code, filename, types);
    }
    return types;
}

function parseFile(code, filename, types) {
    const lines = code.split(/\r?\n/);
    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
        if (lines[lineIndex] !== '#[visited_node]') continue;

        let match;
        while (true) {
            match = lines[++lineIndex].match(/^pub (enum|struct) (.+?)(<'a>)? \{/);
            if (match) break;
        }
        const [, kind, name, lifetimeStr] = match,
            hasLifetime = !!lifetimeStr,
            startLineIndex = lineIndex;

        const itemLines = [];
        while (true) {
            const line = lines[++lineIndex].replace(/\/\/.*$/, '').replace(/\s+/g, ' ').trim();
            if (line === '}') break;
            if (line !== '') itemLines.push(line);
        }

        if (kind === 'struct') {
            types[name] = parseStruct(name, hasLifetime, itemLines, filename, startLineIndex);
        } else {
            types[name] = parseEnum(name, hasLifetime, itemLines, filename, startLineIndex);
        }
    }
}

function parseStruct(name, hasLifetime, lines, filename, startLineIndex) {
    const fields = [];
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        if (line.startsWith('#[')) {
            while (!lines[i].endsWith(']')) {
                i++;
            }
            continue;
        }

        const match = line.match(/^pub ((?:r#)?([a-z_]+)): (.+),$/);
        assert(
            match,
            `Cannot parse line ${startLineIndex + i} in '${filename}' as struct field: '${line}'`
        );
        const [, rawName, name, rawTypeName] = match,
            typeName = rawTypeName.replace(/<'a>/g, '').replace(/<'a, ?/g, '<'),
            {name: innerTypeName, wrappers} = typeAndWrappers(typeName);
        
        fields.push({name, typeName, rawName, rawTypeName, innerTypeName, wrappers});
    }
    return {kind: 'struct', name, hasLifetime, fields};
}

function parseEnum(name, hasLifetime, lines, filename, startLineIndex) {
    const variants = [],
        inherits = [];
    for (const [lineIndex, line] of lines.entries()) {
        const match = line.match(/^(.+?)\((.+?)\)(?: ?= ?(\d+))?,$/);
        if (match) {
            const [, name, rawTypeName, discriminantStr] = match,
                typeName = rawTypeName.replace(/<'a>/g, '').replace(/<'a,\s*/g, '<'),
                {name: innerTypeName, wrappers} = typeAndWrappers(typeName),
                discriminant = discriminantStr ? +discriminantStr : null;
            variants.push({name, typeName, rawTypeName, innerTypeName, wrappers, discriminant});
        } else {
            const match2 = line.match(/^@inherit ([A-Za-z]+)$/);
            assert(
                match2,
                `Cannot parse line ${startLineIndex + lineIndex} in '${filename}' as enum variant: '${line}'`
            );
            inherits.push(match2[1]);
        }
    }
    return {kind: 'enum', name, hasLifetime, variants, inherits};
}
