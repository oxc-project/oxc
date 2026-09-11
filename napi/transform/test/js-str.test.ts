import { expect, test } from "vitest";

import { moduleRunnerTransformSync, transformSync } from "../index";

test.each(["enum", "const enum"])("preserves %s string values", (declaration) => {
  const source = `${declaration} E { A = "\\uD800", B = A + "\\uDC00", C = "\\uFFFDd800" }
    namespace N { export ${declaration} E { A = "\\uDC00", B = A } }
    capture([E.A, E.B, E.C, N.E.A, N.E.B, Object.keys(E), Object.keys(N.E)]);`;
  const result = transformSync("test.ts", source);
  expect(result.errors).toEqual([]);
  let values: unknown;
  // Execute the transformed fixture to check its runtime string values.
  // oxlint-disable-next-line typescript/no-implied-eval
  new Function("capture", result.code)((result: unknown) => {
    values = result;
  });
  expect(values).toEqual([
    "\uD800",
    "\uD800\uDC00",
    "\uFFFDd800",
    "\uDC00",
    "\uDC00",
    ["A", "B", "C"],
    ["A", "B"],
  ]);
});

test("preserves module runner dependency strings", () => {
  const result = moduleRunnerTransformSync(
    "test.js",
    String.raw`import value from "\uD800"; export { value }; import("\uDC00");`,
  );
  expect(result.errors).toEqual([]);
  expect(result.deps).toEqual(["\uD800"]);
  expect(result.dynamicDeps).toEqual(["\uDC00"]);
});

test.each([false, true])(
  "preserves enum member names with lone surrogates (optimize: %s)",
  (optimize) => {
    const source = String.raw`enum E { "\uD800" = "x", "\uFFFDd800" = "marker", A = E["\uD800"] }
    enum E { B = E["\uD800"] }
    const enum C { "\uDC00" = "\uD800", A = C["\uDC00"] + "\uDC00" }
    capture([E.A, E.B, E["\uFFFDd800"], Object.keys(E), C.A]);`;
    const result = transformSync("test.ts", source, {
      typescript: { optimizeConstEnums: optimize, optimizeEnums: optimize },
    });
    expect(result.errors).toEqual([]);
    let values: unknown;
    // Execute the transformed fixture to check its runtime string values.
    // oxlint-disable-next-line typescript/no-implied-eval
    new Function("capture", result.code)((result: unknown) => {
      values = result;
    });
    expect(values).toEqual([
      "x",
      "x",
      "marker",
      ["\uD800", "\uFFFDd800", "A", "B"],
      "\uD800\uDC00",
    ]);
  },
);

test.each(["rewrite", "remove"] as const)(
  "%s import extensions without losing surrogates",
  (mode) => {
    const source =
      'import "./\\uD800.ts"; export * from "./\\uDC00.mts"; import(`./\\uD800.\\u0074s`);';
    const result = transformSync("test.ts", source, {
      typescript: { rewriteImportExtensions: mode },
    });
    expect(result.errors).toEqual([]);
    const modules = moduleRunnerTransformSync("test.js", result.code);
    expect(modules.errors).toEqual([]);
    expect(new Set(modules.deps)).toEqual(
      new Set([
        `./\uD800${mode === "rewrite" ? ".js" : ""}`,
        `./\uDC00${mode === "rewrite" ? ".mjs" : ""}`,
      ]),
    );
    expect(modules.dynamicDeps).toEqual([`./\uD800${mode === "rewrite" ? ".js" : ""}`]);
  },
);

test("preserves numeric JSX entities", () => {
  const result = transformSync("test.jsx", 'capture(<div value="&#xD800;&#xDC00;&#xD800;" />)', {
    jsx: { runtime: "classic" },
  });
  expect(result.errors).toEqual([]);
  let value: unknown;
  // Execute the transformed fixture to check the decoded JSX attribute value.
  // oxlint-disable-next-line typescript/no-implied-eval
  new Function("React", "capture", result.code)(
    { createElement: (_tag: string, props: { value: string }) => props.value },
    (result: unknown) => {
      value = result;
    },
  );
  expect(value).toBe("\uD800\uDC00\uD800");
});
