import { Worker } from 'node:worker_threads';
import { describe, expect, it } from 'vitest';

import { minify } from '../index';

describe('simple', () => {
  const code = '/*! legal comment */\nfunction foo() { var bar; bar(undefined) } foo();';

  it('matches output', () => {
    const ret = minify('test.js', code, { sourcemap: true });
    expect(ret).toStrictEqual({
      'code': 'function foo(){var e;e(void 0)}foo();',
      'map': {
        'mappings': 'AACA,SAAS,KAAM,CAAE,IAAIA,EAAKA,EAAI,OAAY,CAAC',
        'names': [
          'bar',
        ],
        'sources': [
          'test.js',
        ],
        'sourcesContent': [
          code,
        ],
        'version': 3,
      },
      'errors': [],
    });
  });

  it('can turn off everything', () => {
    const ret = minify('test.js', code, { compress: false, mangle: false, codegen: { removeWhitespace: false } });
    expect(ret.code).toBe(
      'function foo() {\n\tvar bar;\n\tbar(undefined);\n}\nfoo();\n',
    );
  });

  it('defaults to esnext', () => {
    const code = 'try { foo } catch (e) {}';
    const ret = minify('test.js', code);
    expect(ret.code).toBe('try{foo}catch{}');
  });

  it('returns parser error', () => {
    const code = 'const';
    const ret = minify('test.js', code);
    expect(ret.code).toBe('');
    expect(ret.errors.length).toBe(1);
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
