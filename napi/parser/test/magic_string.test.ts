import { assert, describe, it } from 'vitest';

import type { StringLiteral, VariableDeclaration } from '../index.js';
import { ParserBuilder } from '../index.js';

describe('simple', () => {
  const code = 'const s: String = "测试"';

  it('calls magic string APIs', () => {
    // `oxc` holds a magic string instance on the Rust side.
    const oxc = new ParserBuilder(code);

    const ast = oxc.parseSync({ sourceFilename: 'test.ts' }).program;
    const declaration = ast.body[0] as VariableDeclaration;
    const stringLiteral = declaration.declarations[0].init as StringLiteral;

    // These spans are in utf8 offsets.
    const start = stringLiteral.start + 1;
    const end = stringLiteral.end - 1;

    // Access source text by utf8 offset.
    assert.equal(oxc.sourceText(start, end), '测试');

    // Magic string manipulation.
    oxc.remove(start, end);
    assert.equal(oxc.toString(), 'const s: String = ""');
  });
});
