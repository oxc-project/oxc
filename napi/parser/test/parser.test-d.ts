import { assertType, describe, it } from 'vitest';

import type { Statement } from '../index';
import * as oxc from '../index';

describe('parse', () => {
  const code = '/* comment */ foo';

  it('checks type', async () => {
    const ret = oxc.parseSync(code);
    assertType<Statement>(ret.program.body[0]);
  });
});
