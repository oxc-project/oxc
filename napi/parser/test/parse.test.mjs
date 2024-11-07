import { describe, expect, it } from 'vitest';

import * as oxc from '../index.js';

describe('parse', () => {
  const code = '/* comment */ foo';

  it('matches output', async () => {
    const ret = oxc.parseSync(code);
    expect(ret.program.body.length).toBe(1);
    expect(ret.errors.length).toBe(0);
    expect(ret.comments.length).toBe(1);

    const comment = ret.comments[0];
    expect(comment).toEqual({
      'type': 'Block',
      'start': 0,
      'end': 13,
      'value': ' comment ',
    });
    expect(code.substring(comment.start, comment.end)).toBe('/*' + comment.value + '*/');

    const ret2 = await oxc.parseAsync(code);
    expect(ret).toEqual(ret2);
  });
});
