import assert from 'assert';
import { parseSync } from '../../npm/parser-wasm/node/oxc_parser_wasm.js';

const code = 'let foo';
const result = parseSync(code, { sourceFilename: 'test.ts' });
assert(result.errors.length === 0);
assert(result.program.body.length === 1);
