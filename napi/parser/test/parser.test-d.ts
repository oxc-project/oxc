import { assertType, describe, it } from 'vitest';

import type { Statement } from '../index';
import { parseSync } from '../index';

describe('parse', () => {
  const code = '/* comment */ foo';

  it('checks type', async () => {
    const ret = parseSync('test.js', code);
    assertType<Statement>(ret.program.body[0]);
  });
});
