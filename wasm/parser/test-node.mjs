import assert from 'assert';
import { parseSync } from '../../npm/parser-wasm/node/oxc_parser_wasm.js';

const code = '/abc/gu; 123n;';
const result = parseSync(code, { sourceFilename: 'test.ts' });

assert(result.errors.length === 0);

// Check `program` getter caches result
const program = result.program;
assert(result.program === program);

// Check output is correct
assert(program.type === 'Program');
assert(program.body.length === 2);

// Check `RegExp`s and `BigInt`s are deserialized correctly
assert(program.body[0].expression.value instanceof RegExp);
assert(typeof program.body[1].expression.value === 'bigint');
