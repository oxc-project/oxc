import { describe, expect, it } from 'vitest';

import { parseAsync, parseSync } from '../index.js';

describe('parse', () => {
  const code = '/* comment */ foo';

  it('uses the `lang` option', () => {
    const ret = parseSync('test.vue', code, { lang: 'ts' });
    expect(ret.program.body.length).toBe(1);
    expect(ret.errors.length).toBe(0);
  });

  it('matches output', async () => {
    const ret = await parseAsync('test.js', code);
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
  });
});

describe('error', () => {
  const code = 'asdf asdf';

  it('returns structured error', () => {
    const ret = parseSync('test.js', code);
    expect(ret.errors.length).toBe(1);
    expect(ret.errors[0]).toStrictEqual({
      'helpMessage': 'Try insert a semicolon here',
      'labels': [
        {
          'end': 4,
          'start': 4,
        },
      ],
      'message': 'Expected a semicolon or an implicit semicolon after a statement, but found none',
      'severity': 'Error',
    });
  });
});
