import {readFile} from 'fs/promises';
import {join as pathJoin} from 'path';
import {fileURLToPath} from 'url';
import assert from 'assert';
import {typeAndWrappers, snakeToCamel} from './utils.mjs';

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
    const lines = code.split(/\r?\n/).map(
        line => line.replace(/\s+/g, ' ').replace(/ ?\/\/.*$/, '')
    );
    for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
        const lineMatch = lines[lineIndex].match(/^#\[visited_node ?([\]\(])/);
        if (!lineMatch) continue;

        let scopeArgs = null;
        if (lineMatch[1] === '(') {
            let line = lines[lineIndex].slice(lineMatch[0].length),
                scopeArgsStr = '';
            while (!line.endsWith(')]')) {
                scopeArgsStr += ` ${line}`;
                line = lines[++lineIndex];
            }
            scopeArgsStr += ` ${line.slice(0, -2)}`;
            scopeArgsStr = scopeArgsStr.trim().replace(/  +/g, ' ').replace(/,$/, '');

            scopeArgs = parseScopeArgs(scopeArgsStr, filename, lineIndex);
        }

        let match;
        while (true) {
            match = lines[++lineIndex].match(/^pub (enum|struct) ((.+?)(?:<'a>)?) \{/);
            if (match) break;
        }
        const [, kind, rawName, name] = match,
            startLineIndex = lineIndex;

        const itemLines = [];
        while (true) {
            const line = lines[++lineIndex].trim();
            if (line === '}') break;
            if (line !== '') itemLines.push(line);
        }

        if (kind === 'struct') {
            types[name] = parseStruct(name, rawName, itemLines, scopeArgs, filename, startLineIndex);
        } else {
            types[name] = parseEnum(name, rawName, itemLines, filename, startLineIndex);
        }
    }
}

function parseStruct(name, rawName, lines, scopeArgs, filename, startLineIndex) {
    const fields = [];
    for (let i = 0; i < lines.length; i++) {
        let line = lines[i];
        const isScopeEntry = line === '#[scope(enter_before)]';
        if (isScopeEntry) {
            line = lines[++i];
        } else if (line.startsWith('#[')) {
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

        if (isScopeEntry) scopeArgs.enterScopeBefore = name;
    }
    return {kind: 'struct', name, rawName, fields, scopeArgs};
}

function parseEnum(name, rawName, lines, filename, startLineIndex) {
    const variants = [],
        inherits = [];
    for (const [lineIndex, line] of lines.entries()) {
        const match = line.match(/^(.+?)\((.+?)\)(?: ?= ?(\d+))?,$/);
        if (match) {
            const [, name, rawTypeName, discriminantStr] = match,
                typeName = rawTypeName.replace(/<'a>/g, '').replace(/<'a, ?/g, '<'),
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
    return {kind: 'enum', name, rawName, variants, inherits};
}

function parseScopeArgs(argsStr, filename, lineIndex) {
    if (!argsStr) return null;

    const matchAndConsume = (regex) => {
        const match = argsStr.match(regex);
        assert(match);
        argsStr = argsStr.slice(match[0].length);
        return match.slice(1);
    };

    const args = {};
    try {
        while (true) {
            const [key] = matchAndConsume(/^([a-z_]+)\(/);
            assert(
                ['scope', 'scope_if', 'strict_if'].includes(key),
                `Unexpected visited_node macro arg: ${key}`
            );

            let bracketCount = 1,
                index = 0;
            for (; index < argsStr.length; index++) {
                const char = argsStr[index];
                if (char === '(') {
                    bracketCount++;
                } else if (char === ')') {
                    bracketCount--;
                    if (bracketCount === 0) break;
                }
            }
            assert(bracketCount === 0);

            const camelKey = key.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
            args[camelKey] = argsStr.slice(0, index).trim();
            argsStr = argsStr.slice(index + 1);
            if (argsStr === '') break;

            matchAndConsume(/^ ?, ?/);
        }

        assert(args.scope, 'Missing key `scope`');
    } catch (err) {
        throw new Error(
            `Cannot parse visited_node args: ${argsStr} in ${filename}:${lineIndex}\n${err?.message}`
        );
    }

    return args;
}
