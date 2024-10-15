import { assert, describe, it } from 'vitest';

import oxc from './index.js';

describe('module lexer', () => {
  const code = 'export { foo }';

  it('matches output', () => {
    const ret = oxc.moduleLexerSync(code);
    assert(ret.exports.length == 1);
  });

  it('matches output async ', async () => {
    const ret = await oxc.moduleLexerAsync(code);
    assert(ret.exports.length == 1);
  });
});
