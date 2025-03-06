import { assert, describe, it } from 'vitest';
import { parseSync } from '../../npm/parser-wasm/node/oxc_parser_wasm.js';

describe('simple', () => {
  it('should parse', () => {
    const code = '/abc/gu; 123n; 1e+350; // 🤨';
    const result = parseSync(code, { sourceFilename: 'test.ts' });

    assert(result.errors.length === 0);

    // Check `program` getter caches result
    const program = result.program;
    assert(result.program === program);

    // Check output is correct
    assert(program.type === 'Program');
    assert(program.body.length === 3);

    // Check `RegExp`s, `BigInt`s and `Infinity` are deserialized correctly
    assert(program.body[0].expression.value instanceof RegExp);
    assert(typeof program.body[1].expression.value === 'bigint');
    const inf = program.body[2].expression.value;
    assert(typeof inf === 'number');
    assert(inf === Infinity);
  });
});
