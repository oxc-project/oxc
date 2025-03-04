import { Worker } from 'node:worker_threads';
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

  it('checks semantic', async () => {
    const code = 'let x; let x;';
    let ret = await parseAsync('test.js', code);
    expect(ret.errors.length).toBe(0);

    ret = await parseAsync('test.js', code, {
      showSemanticErrors: true,
    });
    expect(ret.errors.length).toBe(1);
  });

  describe('TS properties', () => {
    const code = 'let x;';

    const withTsFields = {
      type: 'Program',
      start: 0,
      end: 6,
      body: [
        {
          type: 'VariableDeclaration',
          start: 0,
          end: 6,
          declarations: [
            {
              type: 'VariableDeclarator',
              start: 4,
              end: 5,
              id: {
                type: 'Identifier',
                start: 4,
                end: 5,
                name: 'x',
                typeAnnotation: null,
                optional: false,
              },
              init: null,
              definite: false,
            },
          ],
          kind: 'let',
          declare: false,
        },
      ],
      sourceType: 'module',
      hashbang: null,
    };

    const withoutTsFields = {
      type: 'Program',
      start: 0,
      end: 6,
      body: [
        {
          type: 'VariableDeclaration',
          start: 0,
          end: 6,
          declarations: [
            {
              type: 'VariableDeclarator',
              start: 4,
              end: 5,
              id: {
                type: 'Identifier',
                start: 4,
                end: 5,
                name: 'x',
              },
              init: null,
            },
          ],
          kind: 'let',
        },
      ],
      sourceType: 'module',
      hashbang: null,
    };

    describe('included when', () => {
      it('filename has `.ts` extension', () => {
        const { program } = parseSync('test.ts', code);
        expect(program).toEqual(withTsFields);
      });

      it('`lang` option is "ts"', () => {
        const { program } = parseSync('test.js', code, { lang: 'ts' });
        expect(program).toEqual(withTsFields);
      });

      it('`astType` option is "ts"', () => {
        const { program } = parseSync('test.js', code, { lang: 'js', astType: 'ts' });
        expect(program).toEqual(withTsFields);
      });
    });

    describe('not included when', () => {
      it('filename has `.js` extension', () => {
        const { program } = parseSync('test.js', code);
        expect(program).toEqual(withoutTsFields);
      });

      it('`lang` option is "js"', () => {
        const { program } = parseSync('test.ts', code, { lang: 'js' });
        expect(program).toEqual(withoutTsFields);
      });

      it('`astType` option is "js"', () => {
        const { program } = parseSync('test.ts', code, { lang: 'ts', astType: 'js' });
        expect(program).toEqual(withoutTsFields);
      });
    });
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

  describe('hashbang', () => {
    it('is `null` when no hashbang', () => {
      const ret = parseSync('test.js', 'let x;');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(1);
      expect(ret.program.hashbang).toBeNull();
    });

    it('is defined when hashbang', () => {
      const ret = parseSync('test.js', '#!/usr/bin/env node\nlet x;');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(1);
      expect(ret.program.hashbang).toEqual({
        type: 'Hashbang',
        start: 0,
        end: 19,
        value: '/usr/bin/env node',
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

describe('worker', () => {
  it('should run', async () => {
    const code = await new Promise((resolve, reject) => {
      const worker = new Worker('./test/worker.mjs');
      worker.on('error', (err) => {
        reject(err);
      });
      worker.on('exit', (code) => {
        resolve(code);
      });
    });
    expect(code).toBe(0);
  });
});
