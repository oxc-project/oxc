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

  it('`Infinity` is represented as `Infinity` number', () => {
    const ret = parseSync('test.js', '1e+350');
    expect(ret.errors.length).toBe(0);
    expect(ret.program.body.length).toBe(1);
    expect(ret.program.body[0]).toEqual({
      type: 'ExpressionStatement',
      start: 0,
      end: 6,
      expression: {
        type: 'Literal',
        start: 0,
        end: 6,
        value: Infinity,
        raw: '1e+350',
      },
    });
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

describe('UTF-16 span', () => {
  it('basic', async () => {
    const code = "'🤨'";
    const utf16 = await parseAsync('test.js', code);
    expect(utf16.program.end).toMatchInlineSnapshot(`4`);
  });

  it('comment', async () => {
    const ret = await parseAsync('test.js', `// ∞`);
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
  });

  it('module record', async () => {
    const ret = await parseAsync('test.js', `"🤨";import x from "x"; export { x };import("y");import.meta.z`);
    expect(ret.module).toMatchInlineSnapshot(`
      {
        "dynamicImports": [
          {
            "end": 48,
            "moduleRequest": {
              "end": 47,
              "start": 44,
            },
            "start": 37,
          },
        ],
        "hasModuleSyntax": true,
        "importMetas": [
          {
            "end": 60,
            "start": 49,
          },
        ],
        "staticExports": [
          {
            "end": 23,
            "entries": [
              {
                "end": 34,
                "exportName": {
                  "end": 34,
                  "kind": "Name",
                  "name": "x",
                  "start": 33,
                },
                "importName": {
                  "end": 13,
                  "kind": "Name",
                  "name": "x",
                  "start": 12,
                },
                "localName": {
                  "kind": "None",
                },
                "moduleRequest": {
                  "end": 22,
                  "start": 19,
                  "value": "x",
                },
                "start": 33,
              },
            ],
            "start": 5,
          },
        ],
        "staticImports": [
          {
            "end": 23,
            "entries": [
              {
                "importName": {
                  "end": 13,
                  "kind": "Default",
                  "start": 12,
                },
                "isType": false,
                "localName": {
                  "end": 13,
                  "start": 12,
                  "value": "x",
                },
              },
            ],
            "moduleRequest": {
              "end": 22,
              "start": 19,
              "value": "x",
            },
            "start": 5,
          },
        ],
      }
    `);
  });

  it('error', async () => {
    const ret = await parseAsync('test.js', `"🤨";asdf asdf`);
    expect(ret.errors).toMatchInlineSnapshot(`
      [
        {
          "helpMessage": "Try insert a semicolon here",
          "labels": [
            {
              "end": 9,
              "start": 9,
            },
          ],
          "message": "Expected a semicolon or an implicit semicolon after a statement, but found none",
          "severity": "Error",
        },
      ]
    `);
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
