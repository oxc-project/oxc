import { assert, describe, it } from 'vitest';

import oxc from './index.js';

describe('parse', () => {
  const code = '/* comment */ foo';

  it('matches output', () => {
    const ret = oxc.parseSync(code);
    assert(JSON.parse(ret.program).body.length == 1);
    assert(ret.errors.length == 0);
    assert(ret.comments.length == 1);
  });

  it('matches output async ', async () => {
    const ret = await oxc.parseAsync(code);
    assert(JSON.parse(ret.program).body.length == 1);
    assert(ret.errors.length == 0);
    assert(ret.comments.length == 1);
  });
});
