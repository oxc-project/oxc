import { assert, describe, expect, it } from 'vitest';

import * as oxc from '../index.js';

describe('module lexer', () => {
  const code = 'export { foo }';

  it('matches output', () => {
    const ret = oxc.moduleLexerSync(code);
    assert(ret.exports.length == 1);
  });

  it('matches output async', async () => {
    const ret = await oxc.moduleLexerAsync(code);
    assert(ret.exports.length == 1);
  });

  it('returns export *', async () => {
    const ret = await oxc.moduleLexerAsync("export * from 'foo';");
    expect(ret).toEqual(
      {
        imports: [{ n: 'foo', s: 15, e: 18, ss: 0, se: 20, d: -3, a: -1 }],
        exports: [],
        hasModuleSyntax: true,
        facade: true,
      },
    );
  });
});
