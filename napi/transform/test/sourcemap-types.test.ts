import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { transformSync } from "../index";

// `sources` is typed `Array<string | null>`: ECMA-426 permits `null` for
// unknown source names, but not `undefined`, which is not representable in
// JSON. See https://github.com/oxc-project/oxc/issues/30311 and
// https://tc39.es/ecma426/#sec-source-map-format.
describe("source map types", () => {
  const declarations = ["../index.d.ts", "../transform.wasi.d.cts", "../transform.wasip1.d.cts"];

  it.each(declarations)("%s types `sources` as Array<string | null>", (file) => {
    const contents = readFileSync(join(import.meta.dirname, file), "utf8");
    expect(contents).toContain("sources: Array<string | null>");
    expect(contents).not.toContain("sources: Array<string | undefined | null>");
  });

  it("emits JSON-representable sources", () => {
    const { map } = transformSync("test.ts", "export const A = 1;\n", { sourcemap: true });
    if (!map) throw new Error("source map missing");
    for (const source of map.sources) {
      expect(source).not.toBeUndefined();
      expect(JSON.parse(JSON.stringify(source))).toEqual(source);
    }
  });
});
