import { readFile } from 'fs/promises';
import { join as pathJoin } from 'path';
import { fileURLToPath } from 'url';
import { typeAndWrappers } from './utils.mjs';

const FILENAMES = ['js.rs', 'jsx.rs', 'literal.rs', 'ts.rs'];

/**
 * @typedef {Record<string, StructType | EnumType>} Types
 */

/**
 * @typedef {Object} EnumType
 *
 * @property {'enum'} kind
 * @property {string} name
 * @property {string} rawName
 * @property {Variant[]} variants
 * @property {string[]} inherits
 */

/**
 * @typedef {Object} Variant
 *
 * @property {string} name
 * @property {string} typeName
 * @property {string} rawTypeName
 * @property {string} innerTypeName
 * @property {string[]} wrappers
 * @property {number | null} discriminant
 */

/**
 * @typedef {Object} StructType
 *
 * @property {'struct'} kind
 * @property {string} name
 * @property {string} rawName
 * @property {Field[]} fields
 * @property {ScopeArgs} scopeArgs
 */

/**
 * @typedef {Object} Field
 *
 * @property {string} name
 * @property {string} typeName
 * @property {string} rawName
 * @property {string} rawTypeName
 * @property {string} innerTypeName
 * @property {string[]} wrappers
 */

/**
 * @typedef {Object} ScopeArgs
 *
 * @property {string} flags
 * @property {string | null} strictIf
 * @property {string | null} enterScopeBefore
 * @property {string | null} exitScopeBefore
 */

/**
 * Parse type defs from Rust files.
 *
 * @returns {Promise<Types>}
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

class Position {
  /**
   * @param {string} filename
   * @param {number} index
   */
  constructor(filename, index) {
    /** @type {string} */
    this.filename = filename;
    /** @type {number} */
    this.index = index;
  }

  /**
   * @param {unknown} condition
   * @param {string} [message]
   *
   * @returns {asserts condition}
   */
  assert(condition, message) {
    if (!condition) this.throw(message);
  }
  /**
   * @param {string} [message]
   */
  throw(message) {
    throw new Error(`${message || 'Unknown error'} (at ${this.filename}:${this.index + 1})`);
  }
}

class Lines {
  /**
   * @param {string[]} lines
   * @param {string} filename
   * @param {number} [offset=0]
   */
  constructor(lines, filename, offset = 0) {
    /** @type {string[]} */
    this.lines = lines;

    /** @type {string} */
    this.filename = filename;

    /** @type {number} */
    this.offset = offset;

    /** @type {number} */
    this.index = 0;
  }

  /**
   * @param {string} code
   * @param {string} filename
   *
   * @returns {Lines}
   */
  static fromCode(code, filename) {
    const lines = code.split(/\r?\n/)
      .map(line => line.replace(/\s+/g, ' ').replace(/ ?\/\/.*$/, '').replace(/ $/, ''));
    return new Lines(lines, filename, 0);
  }

  child() {
    return new Lines([], this.filename, this.index);
  }

  current() {
    return this.lines[this.index];
  }

  next() {
    return this.lines[this.index++];
  }

  isEnd() {
    return this.index === this.lines.length;
  }

  position() {
    return new Position(this.filename, this.index + this.offset);
  }

  positionPrevious() {
    return new Position(this.filename, this.index + this.offset - 1);
  }
}

function parseFile(code, filename, types) {
  const lines = Lines.fromCode(code, filename);
  while (!lines.isEnd()) {
    // if not #[ast(visit, ..)]
    if (!/^#\[ast\(.*visit.*\)\]/.test(lines.current())) {
      lines.next();
      continue;
    }

    // Consume attrs and comments, parse `#[scope]` attr
    let match, scopeArgs = null;
    while (!lines.isEnd()) {
      if (/^#\[scope[(\]]/.test(lines.current())) {
        scopeArgs = parseScopeArgs(lines, scopeArgs);
        continue;
      }
      match = lines.next().match(/^pub (enum|struct) ((.+?)(?:<'a>)?) \{/);
      if (match) break;
    }
    lines.position().assert(match, `Could not find enum or struct after #[ast]`);
    const [, kind, rawName, name] = match;

    // Find end of struct / enum
    const itemLines = lines.child();
    while (!lines.isEnd()) {
      const line = lines.next();
      if (line === '}') break;
      itemLines.lines.push(line.trim());
    }

    if (kind === 'struct') {
      types[name] = parseStruct(name, rawName, itemLines, scopeArgs);
    } else {
      types[name] = parseEnum(name, rawName, itemLines);
    }
  }
}

/**
 * @returns {StructType}
 */
function parseStruct(name, rawName, lines, scopeArgs) {
  /** @type {Field[]} */
  const fields = [];

  while (!lines.isEnd()) {
    let isScopeEntry = false, isScopeExit = false, line;
    while (!lines.isEnd()) {
      line = lines.next();
      if (line === '') continue;
      if (line === '#[scope(enter_before)]') {
        isScopeEntry = true;
      } else if (line === '#[scope(exit_before)]') {
        isScopeExit = true;
      } else if (line.startsWith('#[')) {
        while (!line.endsWith(']')) {
          line = lines.next();
        }
      } else {
        break;
      }
    }

    const match = line.match(/^pub ((?:r#)?([a-z_]+)): (.+),$/);
    lines.positionPrevious().assert(match, `Cannot parse line as struct field: '${line}'`);
    const [, rawName, name, rawTypeName] = match,
      typeName = rawTypeName.replace(/<'a>/g, '').replace(/<'a, ?/g, '<'),
      { name: innerTypeName, wrappers } = typeAndWrappers(typeName);

    fields.push({ name, typeName, rawName, rawTypeName, innerTypeName, wrappers });

    if (isScopeEntry) scopeArgs.enterScopeBefore = name;
    if (isScopeExit) scopeArgs.exitScopeBefore = name;
  }
  return { kind: 'struct', name, rawName, fields, scopeArgs };
}

/**
 * @returns {EnumType}
 */
function parseEnum(name, rawName, lines) {
  /** @type {Variant[]} */
  const variants = [];
  /** @type {string[]} */
  const inherits = [];

  while (!lines.isEnd()) {
    let line = lines.next();
    if (line === '') continue;
    if (line.startsWith('#[')) {
      while (!line.endsWith(']')) {
        line = lines.next();
      }
      continue;
    }

    const match = line.match(/^(.+?)\((.+?)\)(?: ?= ?(\d+))?,$/);
    if (match) {
      const [, name, rawTypeName, discriminantStr] = match,
        typeName = rawTypeName.replace(/<'a>/g, '').replace(/<'a, ?/g, '<'),
        { name: innerTypeName, wrappers } = typeAndWrappers(typeName),
        discriminant = discriminantStr ? +discriminantStr : null;
      variants.push({ name, typeName, rawTypeName, innerTypeName, wrappers, discriminant });
    } else {
      const match2 = line.match(/^@inherit ([A-Za-z]+)$/);
      lines.positionPrevious().assert(match2, `Cannot parse line as enum variant: '${line}'`);
      inherits.push(match2[1]);
    }
  }
  return { kind: 'enum', name, rawName, variants, inherits };
}

function parseScopeArgs(lines, scopeArgs) {
  const position = lines.position();

  // Get whole of `#[scope]` attr text as a single line string
  let scopeArgsStr = '';
  let line = lines.next();
  if (line !== '#[scope]') {
    line = line.slice('#[scope('.length);
    while (!line.endsWith(')]')) {
      scopeArgsStr += ` ${line}`;
      line = lines.next();
    }
    scopeArgsStr += ` ${line.slice(0, -2)}`;
    scopeArgsStr = scopeArgsStr.trim().replace(/  +/g, ' ').replace(/,$/, '');
  }

  // Parse attr
  return parseScopeArgsStr(scopeArgsStr, scopeArgs, position);
}

const SCOPE_ARGS_KEYS = { flags: 'flags', strict_if: 'strictIf' };

/**
 * @param {string} argsStr
 * @param {ScopeArgs| null} args
 * @param {Position} position
 *
 * @returns {ScopeArgs}
 */
function parseScopeArgsStr(argsStr, args, position) {
  if (!args) {
    args = {
      flags: 'ScopeFlags::empty()',
      strictIf: null,
      enterScopeBefore: null,
      exitScopeBefore: null,
    };
  }

  if (!argsStr) return args;

  /** @param {RegExp} regex */
  const matchAndConsume = (regex) => {
    const match = argsStr.match(regex);
    position.assert(match);
    argsStr = argsStr.slice(match[0].length);
    return match.slice(1);
  };

  try {
    while (true) {
      const [keyRaw] = matchAndConsume(/^([a-z_]+)\(/);
      const key = SCOPE_ARGS_KEYS[keyRaw];
      position.assert(key, `Unexpected scope macro arg: ${key}`);

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
      position.assert(bracketCount === 0);

      args[key] = argsStr.slice(0, index).trim();
      argsStr = argsStr.slice(index + 1);
      if (argsStr === '') break;

      matchAndConsume(/^ ?, ?/);
    }
  } catch (err) {
    position.throw(`Cannot parse scope args: '${argsStr}': ${err?.message || 'Unknown error'}`);
  }

  return args;
}
