import { describe, expect, it } from 'vitest';

import { minify } from '../index';

describe('simple', () => {
  const code = 'function foo() { var bar; bar(undefined) } foo();';

  it('matches output', () => {
    const ret = minify('test.js', code, { sourcemap: true });
    expect(ret).toStrictEqual({
      'code': 'function foo(){var e;e(void 0)}foo();',
      'map': {
        'mappings': 'AAAA,SAAS,KAAM,CAAE,IAAIA,EAAK,SAAc,AAAE,CAAC,KAAK',
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
    });
  });

  it('can turn off everything', () => {
    const ret = minify('test.js', code, { compress: false, mangle: false, codegen: { whitespace: false } });
    expect(ret).toStrictEqual({
      'code': 'function foo() {\n\tvar bar;\n\tbar(undefined);\n}\nfoo();\n',
    });
  });
});
