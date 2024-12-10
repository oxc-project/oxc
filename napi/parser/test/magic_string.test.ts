import { describe, expect, it } from 'vitest';

import type { StringLiteral, VariableDeclaration } from '../index.js';
import { parseSync } from '../index.js';

describe('simple', () => {
  const code = 'const s: String = "测试"';

  it('calls magic string APIs', () => {
    // `oxc` holds a magic string instance on the Rust side.
    const { program, magicString: ms } = parseSync('test.ts', code);
    const declaration = program.body[0] as VariableDeclaration;
    const stringLiteral = declaration.declarations[0].init as StringLiteral;

    // These spans are in utf8 offsets.
    const start = stringLiteral.start + 1;
    const end = stringLiteral.end - 1;

    // Access source text by utf8 offset.
    expect(ms.getSourceText(start, end)).toEqual('测试');

    // Access line and column number from utf8 offset.
    expect(ms.getLineColumnNumber(start)).toStrictEqual({
      line: 0,
      column: 19,
    });

    // Magic string manipulation.
    ms.remove(start, end).append(';');
    expect(ms.toString()).toEqual('const s: String = "";');
  });
});
