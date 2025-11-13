import { Worker } from 'node:worker_threads';
import { describe, expect, it, test } from 'vitest';

import { parseAsync, parseSync } from '../src-js/index.js';
import type {
  ExpressionStatement,
  ParserOptions,
  TSTypeAliasDeclaration,
  VariableDeclaration,
} from '../src-js/index.js';

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
      type: 'Block',
      start: 0,
      end: 13,
      value: ' comment ',
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

  describe('sets lang and sourceType', () => {
    const code = 'await(1)';
    let langs: ParserOptions['lang'][] = ['js', 'ts', 'jsx', 'tsx'];
    test.each(langs)('%s', (lang) => {
      const ret = parseSync('test.cjs', code, { lang, sourceType: 'script' });
      expect(ret.errors.length).toBe(0);
      // Parsed as `await(1)`
      expect((ret.program.body[0] as ExpressionStatement).expression.type).toBe('CallExpression');
    });
    test.each(langs)('%s', (lang) => {
      const ret = parseSync('test.cjs', code, { lang, sourceType: 'module' });
      expect(ret.errors.length).toBe(0);
      // Parsed as `await 1`
      expect((ret.program.body[0] as ExpressionStatement).expression.type).toBe('AwaitExpression');
    });
    test('sets lang as dts', () => {
      const code = 'declare const foo';
      const ret = parseSync('test', code, { lang: 'dts' });
      expect(ret.errors.length).toBe(0);
      expect((ret.program.body[0] as VariableDeclaration).declare).toBe(true);
    });
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
                decorators: [],
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

  describe('`StringLiteral`', () => {
    it('lone surrogates', () => {
      const ret = parseSync('test.js', ';"\\uD800\\uDBFF";');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(2);
      expect(ret.program.body[1]).toEqual({
        type: 'ExpressionStatement',
        start: 1,
        end: 16,
        expression: {
          type: 'Literal',
          start: 1,
          end: 15,
          value: '\ud800\udbff',
          raw: '"\\uD800\\uDBFF"',
        },
      });
    });

    it('lossy replacement character', () => {
      const ret = parseSync('test.js', ';"�\\u{FFFD}";');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(2);
      expect(ret.program.body[1]).toEqual({
        type: 'ExpressionStatement',
        start: 1,
        end: 13,
        expression: {
          type: 'Literal',
          start: 1,
          end: 12,
          value: '��',
          raw: '"�\\u{FFFD}"',
        },
      });
    });

    it('lone surrogates and lossy replacement characters', () => {
      const ret = parseSync('test.js', ';"�\\u{FFFD}\\uD800\\uDBFF�\\u{FFFD}";');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(2);
      expect(ret.program.body[1]).toEqual({
        type: 'ExpressionStatement',
        start: 1,
        end: 34,
        expression: {
          type: 'Literal',
          start: 1,
          end: 33,
          value: '��\ud800\udbff��',
          raw: '"�\\u{FFFD}\\uD800\\uDBFF�\\u{FFFD}"',
        },
      });
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

  describe('`TemplateLiteral`', () => {
    it('lone surrogates', () => {
      const ret = parseSync('test.js', '`\\uD800\\uDBFF${x}\\uD800\\uDBFF`;');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(1);
      expect(ret.program.body[0]).toEqual({
        type: 'ExpressionStatement',
        start: 0,
        end: 31,
        expression: {
          type: 'TemplateLiteral',
          start: 0,
          end: 30,
          expressions: [
            {
              type: 'Identifier',
              start: 15,
              end: 16,
              name: 'x',
            },
          ],
          quasis: [
            {
              type: 'TemplateElement',
              start: 1,
              end: 13,
              value: {
                raw: '\\uD800\\uDBFF',
                cooked: '\ud800\udbff',
              },
              tail: false,
            },
            {
              type: 'TemplateElement',
              start: 17,
              end: 29,
              value: {
                raw: '\\uD800\\uDBFF',
                cooked: '\ud800\udbff',
              },
              tail: true,
            },
          ],
        },
      });
    });

    describe('`ImportDeclaration`', () => {
      describe('import defer', () => {
        it('ESTree', () => {
          const ret = parseSync('test.js', 'import defer * as ns from "x";');
          expect(ret.errors.length).toBe(0);
          expect(ret.program.body.length).toBe(1);
          expect(ret.program.body[0]).toEqual({
            type: 'ImportDeclaration',
            start: 0,
            end: 30,
            specifiers: [
              {
                type: 'ImportNamespaceSpecifier',
                start: 13,
                end: 20,
                local: { type: 'Identifier', start: 18, end: 20, name: 'ns' },
              },
            ],
            source: { type: 'Literal', start: 26, end: 29, value: 'x', raw: '"x"' },
            attributes: [],
            phase: 'defer',
          });
        });

        it('TS-ESTree', () => {
          const ret = parseSync('test.ts', 'import defer * as ns from "x";');
          expect(ret.errors.length).toBe(0);
          expect(ret.program.body.length).toBe(1);
          expect(ret.program.body[0]).toEqual({
            type: 'ImportDeclaration',
            start: 0,
            end: 30,
            specifiers: [
              {
                type: 'ImportNamespaceSpecifier',
                start: 13,
                end: 20,
                local: {
                  type: 'Identifier',
                  start: 18,
                  end: 20,
                  decorators: [],
                  name: 'ns',
                  optional: false,
                  typeAnnotation: null,
                },
              },
            ],
            source: { type: 'Literal', start: 26, end: 29, value: 'x', raw: '"x"' },
            attributes: [],
            phase: 'defer',
            importKind: 'value',
          });
        });
      });

      describe('import source', () => {
        it('ESTree', () => {
          const ret = parseSync('test.js', 'import source src from "x";');
          expect(ret.errors.length).toBe(0);
          expect(ret.program.body.length).toBe(1);
          expect(ret.program.body[0]).toEqual({
            type: 'ImportDeclaration',
            start: 0,
            end: 27,
            specifiers: [
              {
                type: 'ImportDefaultSpecifier',
                start: 14,
                end: 17,
                local: { type: 'Identifier', start: 14, end: 17, name: 'src' },
              },
            ],
            source: { type: 'Literal', start: 23, end: 26, value: 'x', raw: '"x"' },
            attributes: [],
            phase: 'source',
          });
        });

        it('TS-ESTree', () => {
          const ret = parseSync('test.ts', 'import source src from "x";');
          expect(ret.errors.length).toBe(0);
          expect(ret.program.body.length).toBe(1);
          expect(ret.program.body[0]).toEqual({
            type: 'ImportDeclaration',
            start: 0,
            end: 27,
            specifiers: [
              {
                type: 'ImportDefaultSpecifier',
                start: 14,
                end: 17,
                local: {
                  type: 'Identifier',
                  start: 14,
                  end: 17,
                  decorators: [],
                  name: 'src',
                  optional: false,
                  typeAnnotation: null,
                },
              },
            ],
            source: { type: 'Literal', start: 23, end: 26, value: 'x', raw: '"x"' },
            attributes: [],
            phase: 'source',
            importKind: 'value',
          });
        });
      });

      describe('`ImportExpression`', () => {
        describe('import.defer()', () => {
          it('ESTree', () => {
            const ret = parseSync('test.js', 'import.defer("x");');
            expect(ret.errors.length).toBe(0);
            expect(ret.program.body.length).toBe(1);
            expect(ret.program.body[0].expression).toEqual({
              type: 'ImportExpression',
              start: 0,
              end: 17,
              source: { type: 'Literal', start: 13, end: 16, value: 'x', raw: '"x"' },
              options: null,
              phase: 'defer',
            });
          });

          // This does *not* align with TS-ESLint.
          // See https://github.com/oxc-project/oxc/pull/11193.
          it('TS-ESTree', () => {
            const ret = parseSync('test.ts', 'import.defer("x");');
            expect(ret.errors.length).toBe(0);
            expect(ret.program.body.length).toBe(1);
            expect(ret.program.body[0].expression).toEqual({
              type: 'ImportExpression',
              start: 0,
              end: 17,
              source: { type: 'Literal', start: 13, end: 16, value: 'x', raw: '"x"' },
              options: null,
              phase: 'defer',
            });
          });
        });

        describe('import.source()', () => {
          it('ESTree', () => {
            const ret = parseSync('test.js', 'import.source("x");');
            expect(ret.errors.length).toBe(0);
            expect(ret.program.body.length).toBe(1);
            expect(ret.program.body[0].expression).toEqual({
              type: 'ImportExpression',
              start: 0,
              end: 18,
              source: { type: 'Literal', start: 14, end: 17, value: 'x', raw: '"x"' },
              options: null,
              phase: 'source',
            });
          });

          // This does *not* align with TS-ESLint.
          // See https://github.com/oxc-project/oxc/pull/11193.
          it('TS-ESTree', () => {
            const ret = parseSync('test.ts', 'import.source("x");');
            expect(ret.errors.length).toBe(0);
            expect(ret.program.body.length).toBe(1);
            expect(ret.program.body[0].expression).toEqual({
              type: 'ImportExpression',
              start: 0,
              end: 18,
              source: { type: 'Literal', start: 14, end: 17, value: 'x', raw: '"x"' },
              options: null,
              phase: 'source',
            });
          });
        });
      });
    });

    it('lossy replacement character', () => {
      const ret = parseSync('test.js', '`�\\u{FFFD}${x}�\\u{FFFD}`;');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(1);
      expect(ret.program.body[0]).toEqual({
        type: 'ExpressionStatement',
        start: 0,
        end: 25,
        expression: {
          type: 'TemplateLiteral',
          start: 0,
          end: 24,
          expressions: [
            {
              type: 'Identifier',
              start: 12,
              end: 13,
              name: 'x',
            },
          ],
          quasis: [
            {
              type: 'TemplateElement',
              start: 1,
              end: 10,
              value: {
                raw: '�\\u{FFFD}',
                cooked: '��',
              },
              tail: false,
            },
            {
              type: 'TemplateElement',
              start: 14,
              end: 23,
              value: {
                raw: '�\\u{FFFD}',
                cooked: '��',
              },
              tail: true,
            },
          ],
        },
      });
    });

    it('lone surrogates and lossy replacement characters', () => {
      const ret = parseSync('test.js', '`�\\u{FFFD}\\uD800${x}\\uDBFF�\\u{FFFD}`;');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(1);
      expect(ret.program.body[0]).toEqual({
        type: 'ExpressionStatement',
        start: 0,
        end: 37,
        expression: {
          type: 'TemplateLiteral',
          start: 0,
          end: 36,
          expressions: [
            {
              type: 'Identifier',
              start: 18,
              end: 19,
              name: 'x',
            },
          ],
          quasis: [
            {
              type: 'TemplateElement',
              start: 1,
              end: 16,
              value: {
                raw: '�\\u{FFFD}\\uD800',
                cooked: '��\ud800',
              },
              tail: false,
            },
            {
              type: 'TemplateElement',
              start: 20,
              end: 35,
              value: {
                raw: '\\uDBFF�\\u{FFFD}',
                cooked: '\udbff��',
              },
              tail: true,
            },
          ],
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
      expect(ret.comments).toHaveLength(0);
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

    it('is also recorded as a line comment in JS files', () => {
      const ret = parseSync('test.js', '#!/usr/bin/env node\nlet x; // foo');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(1);
      expect(ret.comments).toHaveLength(2);
      expect(ret.comments[0]).toEqual({
        type: 'Line',
        start: 0,
        end: 19,
        value: '/usr/bin/env node',
      });
    });

    it('is not recorded as a line comment in TS files', () => {
      const ret = parseSync('test.ts', '#!/usr/bin/env node\nlet x; // foo');
      expect(ret.errors.length).toBe(0);
      expect(ret.program.body.length).toBe(1);
      expect(ret.comments).toHaveLength(1);
      expect(ret.comments[0].value).toBe(' foo');
    });
  });

  describe('preserveParens', () => {
    it('should include parens when true', () => {
      let ret = parseSync('test.js', '(x)');
      expect((ret.program.body[0] as ExpressionStatement).expression.type).toBe('ParenthesizedExpression');

      ret = parseSync('test.ts', 'type Foo = (x)');
      expect((ret.program.body[0] as TSTypeAliasDeclaration).typeAnnotation.type).toBe('TSParenthesizedType');
    });

    it('should omit parens when false', () => {
      const options = { preserveParens: false };
      let ret = parseSync('test.js', '(x)', options);
      expect((ret.program.body[0] as ExpressionStatement).expression.type).toBe('Identifier');

      ret = parseSync('test.ts', 'type Foo = (x)', options);
      expect((ret.program.body[0] as TSTypeAliasDeclaration).typeAnnotation.type).toBe('TSTypeReference');
    });
  });

  describe('ranges', () => {
    it('should include range when true', () => {
      const ret = parseSync('test.js', '(x)', { range: true });
      expect(ret.program.body[0].start).toBe(0);
      expect(ret.program.body[0].range).toEqual([0, 3]);
    });

    it('should not include range when false', () => {
      const ret = parseSync('test.js', '(x)', { range: false });
      expect(ret.program.body[0].range).toBeUndefined();
    });

    it('should not include range by default', () => {
      const ret = parseSync('test.js', '(x)');
      expect(ret.program.body[0].range).toBeUndefined();
    });
  });

  describe('loc', () => {
    it('should include loc when true', () => {
      const ret = parseSync('test.js', 'let x = 1;', { loc: true });
      expect(ret.program.body[0].start).toBe(0);
      expect(ret.program.body[0].end).toBe(10);
      expect(ret.program.body[0].loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 1, column: 10 },
      });
    });

    it('should not include loc when false', () => {
      const ret = parseSync('test.js', 'let x = 1;', { loc: false });
      expect(ret.program.body[0].loc).toBeUndefined();
    });

    it('should not include loc by default', () => {
      const ret = parseSync('test.js', 'let x = 1;');
      expect(ret.program.body[0].loc).toBeUndefined();
    });

    it('should handle multiline code correctly', () => {
      const code = `let x = 1;
let y = 2;
let z = 3;`;
      const ret = parseSync('test.js', code, { loc: true });

      // First declaration: let x = 1;
      expect(ret.program.body[0].loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 1, column: 10 },
      });

      // Second declaration: let y = 2;
      expect(ret.program.body[1].loc).toEqual({
        start: { line: 2, column: 0 },
        end: { line: 2, column: 10 },
      });

      // Third declaration: let z = 3;
      expect(ret.program.body[2].loc).toEqual({
        start: { line: 3, column: 0 },
        end: { line: 3, column: 10 },
      });
    });

    it('should handle nested nodes correctly', () => {
      const code = 'function foo() {\n  return 42;\n}';
      const ret = parseSync('test.js', code, { loc: true });

      const functionDecl = ret.program.body[0];
      expect(functionDecl.loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 3, column: 1 },
      });

      const returnStmt = functionDecl.body.body[0];
      expect(returnStmt.loc).toEqual({
        start: { line: 2, column: 2 },
        end: { line: 2, column: 11 },
      });
    });

    it('should work with both range and loc options', () => {
      const ret = parseSync('test.js', 'let x = 1;', { range: true, loc: true });

      expect(ret.program.body[0].range).toEqual([0, 10]);
      expect(ret.program.body[0].loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 1, column: 10 },
      });
    });

    it('should handle Unicode characters correctly', () => {
      const code = 'let 🤨 = "hello";';
      const ret = parseSync('test.js', code, { loc: true });

      expect(ret.program.body[0].loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 1, column: 17 },
      });
    });

    it('should handle different line endings', () => {
      // Test with \r\n line endings
      const codeWindows = 'let x = 1;\r\nlet y = 2;';
      const retWindows = parseSync('test.js', codeWindows, { loc: true });

      expect(retWindows.program.body[0].loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 1, column: 10 },
      });
      expect(retWindows.program.body[1].loc).toEqual({
        start: { line: 2, column: 0 },
        end: { line: 2, column: 10 },
      });

      // Test with \r line endings (old Mac)
      const codeMac = 'let x = 1;\rlet y = 2;';
      const retMac = parseSync('test.js', codeMac, { loc: true });

      expect(retMac.program.body[0].loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 1, column: 10 },
      });
      expect(retMac.program.body[1].loc).toEqual({
        start: { line: 2, column: 0 },
        end: { line: 2, column: 10 },
      });
    });

    it('should handle expressions and identifiers correctly', () => {
      const code = 'const result = foo + bar;';
      const ret = parseSync('test.js', code, { loc: true });

      const varDecl = ret.program.body[0];
      const declarator = varDecl.declarations[0];

      // Identifier 'result'
      expect(declarator.id.loc).toEqual({
        start: { line: 1, column: 6 },
        end: { line: 1, column: 12 },
      });

      // Binary expression 'foo + bar'
      expect(declarator.init.loc).toEqual({
        start: { line: 1, column: 15 },
        end: { line: 1, column: 24 },
      });

      // Left identifier 'foo'
      expect(declarator.init.left.loc).toEqual({
        start: { line: 1, column: 15 },
        end: { line: 1, column: 18 },
      });

      // Right identifier 'bar'
      expect(declarator.init.right.loc).toEqual({
        start: { line: 1, column: 21 },
        end: { line: 1, column: 24 },
      });
    });

    it('should handle empty lines correctly', () => {
      const code = 'let x = 1;\n\nlet y = 2;';
      const ret = parseSync('test.js', code, { loc: true });

      expect(ret.program.body[0].loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 1, column: 10 },
      });

      // Second statement should be on line 3 (skipping empty line 2)
      expect(ret.program.body[1].loc).toEqual({
        start: { line: 3, column: 0 },
        end: { line: 3, column: 10 },
      });
    });

    it('should handle comments with loc', () => {
      const code = '/* comment */ let x = 1;';
      const ret = parseSync('test.js', code, { loc: true });

      // Variable declaration should start after the comment
      expect(ret.program.body[0].loc).toEqual({
        start: { line: 1, column: 14 },
        end: { line: 1, column: 24 },
      });

      // Comment should also have loc
      expect(ret.comments[0].loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 1, column: 13 },
      });
    });

    it('should handle TypeScript specific nodes with loc', () => {
      const code = 'interface User {\n  name: string;\n  age: number;\n}';
      const ret = parseSync('test.ts', code, { loc: true });

      const interfaceDecl = ret.program.body[0];
      expect(interfaceDecl.loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 4, column: 1 },
      });
    });

    it('should handle complex expressions with correct column positions', () => {
      const code = 'const obj = { foo: bar.baz().qux, hello: "world" };';
      const ret = parseSync('test.js', code, { loc: true });

      const varDecl = ret.program.body[0];
      const declarator = varDecl.declarations[0];

      // Object expression
      expect(declarator.init.loc).toEqual({
        start: { line: 1, column: 12 },
        end: { line: 1, column: 51 },
      });

      // First property
      expect(declarator.init.properties[0].loc).toEqual({
        start: { line: 1, column: 14 },
        end: { line: 1, column: 33 },
      });
    });

    it('should handle string literals with escapes', () => {
      const code = 'const str = "hello\\nworld";';
      const ret = parseSync('test.js', code, { loc: true });

      const varDecl = ret.program.body[0];
      const declarator = varDecl.declarations[0];

      // String literal
      expect(declarator.init.loc).toEqual({
        start: { line: 1, column: 12 },
        end: { line: 1, column: 26 },
      });
    });

    it('should handle template literals across multiple lines', () => {
      const code = 'const template = `line1\nline2\nline3`;';
      const ret = parseSync('test.js', code, { loc: true });

      const varDecl = ret.program.body[0];
      const declarator = varDecl.declarations[0];

      // Template literal spanning multiple lines
      expect(declarator.init.loc).toEqual({
        start: { line: 1, column: 17 },
        end: { line: 3, column: 6 },
      });
    });

    it('should handle zero-length nodes correctly', () => {
      const code = 'for (;;) {}';
      const ret = parseSync('test.js', code, { loc: true });

      const forStmt = ret.program.body[0];
      expect(forStmt.loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 1, column: 11 },
      });
    });

    it('should handle arrow functions with different syntaxes', () => {
      const code = `const fn1 = () => 42;
const fn2 = (x) => {
  return x * 2;
};`;
      const ret = parseSync('test.js', code, { loc: true });

      // First arrow function (single expression)
      const firstDecl = ret.program.body[0];
      expect(firstDecl.declarations[0].init.loc).toEqual({
        start: { line: 1, column: 12 },
        end: { line: 1, column: 21 },
      });

      // Second arrow function (block body)
      const secondDecl = ret.program.body[1];
      expect(secondDecl.declarations[0].init.loc).toEqual({
        start: { line: 2, column: 12 },
        end: { line: 4, column: 1 },
      });
    });

    it('should handle async functions correctly', () => {
      const code = 'async function fetchData() {\n  return await fetch("/api");\n}';
      const ret = parseSync('test.js', code, { loc: true });

      const asyncFn = ret.program.body[0];
      expect(asyncFn.loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 3, column: 1 },
      });

      // Return statement with await expression
      const returnStmt = asyncFn.body.body[0];
      expect(returnStmt.loc).toEqual({
        start: { line: 2, column: 2 },
        end: { line: 2, column: 26 },
      });
    });

    it('should work with parseAsync as well', async () => {
      const code = 'let x = 1;\nlet y = 2;';
      const ret = await parseAsync('test.js', code, { loc: true });

      expect(ret.program.body[0].loc).toEqual({
        start: { line: 1, column: 0 },
        end: { line: 1, column: 10 },
      });

      expect(ret.program.body[1].loc).toEqual({
        start: { line: 2, column: 0 },
        end: { line: 2, column: 10 },
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
                "isType": false,
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
          "codeframe": "
        x Expected a semicolon or an implicit semicolon after a statement, but found
        | none
         ,-[test.js:1:12]
       1 | "🤨";asdf asdf
         :          ^
         \`----
        help: Try inserting a semicolon here
      ",
          "helpMessage": "Try inserting a semicolon here",
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
    delete ret.errors[0].codeframe;
    expect(ret.errors[0]).toStrictEqual({
      helpMessage: 'Try inserting a semicolon here',
      labels: [
        {
          end: 4,
          start: 4,
        },
      ],
      message: 'Expected a semicolon or an implicit semicolon after a statement, but found none',
      severity: 'Error',
    });
  });
});

describe('worker', () => {
  it('should run', async () => {
    const code = await new Promise((resolve, reject) => {
      const worker = new Worker('./test/worker.js');
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
