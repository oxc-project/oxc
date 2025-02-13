import { assert, describe, expect, it } from 'vitest';

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

  it('`BigIntLiteral` has `value` as `BigInt`', () => {
    const ret = parseSync('test.js', '123_456n');
    expect(ret.errors.length).toBe(0);
    expect(ret.program.body.length).toBe(1);
    expect(ret.program.body[0]).toEqual({
      type: 'ExpressionStatement',
      start: 0,
      end: 8,
      expression: {
        type: 'Literal',
        start: 0,
        end: 8,
        value: 123456n,
        raw: '123_456n',
        bigint: '123456',
      },
    });
  });

  describe('`RegExpLiteral`', () => {
    it('has `value` as `RegExp` when valid regexp', () => {
      const ret = parseSync('test.js', '/abc/gu');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(1);
      expect(ret.program.body[0]).toEqual({
        type: 'ExpressionStatement',
        start: 0,
        end: 7,
        expression: {
          type: 'Literal',
          start: 0,
          end: 7,
          value: /abc/gu,
          raw: '/abc/gu',
          regex: {
            pattern: 'abc',
            flags: 'gu',
          },
        },
      });
    });

    it('has `value` as `null` when invalid regexp', () => {
      const ret = parseSync('test.js', '/+/');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(1);
      expect(ret.program.body[0]).toEqual({
        type: 'ExpressionStatement',
        start: 0,
        end: 3,
        expression: {
          type: 'Literal',
          start: 0,
          end: 3,
          value: null,
          raw: '/+/',
          regex: {
            pattern: '+',
            flags: '',
          },
        },
      });
    });
  });
});

it('utf16 span', async () => {
  const code = "'🤨'";
  {
    const ret = await parseAsync('test.js', code);
    expect(ret.program.end).toMatchInlineSnapshot(`6`);
  }
  {
    const ret = await parseAsync('test.js', code, {
      convertSpanUtf16: true,
    });
    expect(ret.program.end).toMatchInlineSnapshot(`4`);
  }
  {
    const code = `// ∞`;
    const ret = await parseAsync('test.js', code, {
      convertSpanUtf16: true,
    });
    expect(ret.comments).toMatchInlineSnapshot(`
      [
        {
          "end": 4,
          "start": 0,
          "type": "Line",
          "value": " ∞",
        },
      ]
    `);
  }
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
