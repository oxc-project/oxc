import { describe, expect, test } from "vitest";
import { moduleRunnerTransform, moduleRunnerTransformSync } from "../index";

describe("moduleRunnerTransformSync", () => {
  test("dynamic import", async () => {
    const result = moduleRunnerTransformSync("index.js", `export const i = () => import('./foo')`);
    expect(result?.code).toMatchInlineSnapshot(`
      "Object.defineProperty(__vite_ssr_exports__, "i", {
      	enumerable: true,
      	configurable: true,
      	get() {
      		return i;
      	}
      });
      const i = () => __vite_ssr_dynamic_import__("./foo");
      "
    `);
    expect(result?.deps).toEqual([]);
    expect(result?.dynamicDeps).toEqual(["./foo"]);
  });

  test("sourcemap", async () => {
    const map = moduleRunnerTransformSync("index.js", `export const a = 1`, {
      sourcemap: true,
    })?.map;

    expect(map).toMatchInlineSnapshot(`
      {
        "mappings": ";;;;;;;AAAO,MAAM,IAAI",
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

describe("moduleRunnerTransform", () => {
  test("produces same result as sync", async () => {
    const syncResult = moduleRunnerTransformSync("index.js", `export const a = 1`);
    const asyncResult = await moduleRunnerTransform("index.js", `export const a = 1`);

    expect(asyncResult.code).toEqual(syncResult.code);
    expect(asyncResult.deps).toEqual(syncResult.deps);
    expect(asyncResult.dynamicDeps).toEqual(syncResult.dynamicDeps);
    expect(asyncResult.errors.length).toBe(syncResult.errors.length);
  });
});
