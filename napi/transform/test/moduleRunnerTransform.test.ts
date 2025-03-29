import { describe, expect, test } from 'vitest';
import { moduleRunnerTransform } from '../index';

describe('moduleRunnerTransform', () => {
  test('dynamic import', async () => {
    const result = await moduleRunnerTransform('index.js', `export const i = () => import('./foo')`);
    expect(result?.code).toMatchInlineSnapshot(`
			"const i = () => __vite_ssr_dynamic_import__("./foo");
			Object.defineProperty(__vite_ssr_exports__, "i", {
				enumerable: true,
				configurable: true,
				get() {
					return i;
				}
			});
			"
		`);
    expect(result?.deps).toEqual([]);
    expect(result?.dynamicDeps).toEqual(['./foo']);
  });

  test('sourcemap', async () => {
    const map = (
      moduleRunnerTransform(
        'index.js',
        `export const a = 1`,
        {
          sourcemap: true,
        },
      )
    )?.map;

    expect(map).toMatchInlineSnapshot(`
      {
        "mappings": "AAAO,MAAM,IAAI;AAAjB",
        "names": [],
        "sources": [
          "index.js",
        ],
        "sourcesContent": [
          "export const a = 1",
        ],
        "version": 3,
      }
    `);
  });
});
