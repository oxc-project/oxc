import { expect, test } from "vitest";

import { moduleRunnerTransformSync, transformSync } from "../index";

test("preserves module runner dependency strings", () => {
  const result = moduleRunnerTransformSync(
    "test.js",
    String.raw`import value from "\uD800"; export { value }; import("\uDC00");`,
  );
  expect(result.errors).toEqual([]);
  expect(result.deps).toEqual(["\uD800"]);
  expect(result.dynamicDeps).toEqual(["\uDC00"]);
});

test("preserves string values through transformation and codegen", () => {
  const source = String.raw`capture(["\uD800", "\uDC00", "\uD800\uDC00", "\uFFFDd800", "漢😎"]);`;
  const result = transformSync("test.js", source);
  expect(result.errors).toEqual([]);
  let values: unknown;
  // oxlint-disable-next-line typescript/no-implied-eval -- Execute generated code to check UTF-16 values.
  new Function("capture", result.code)((result: unknown) => {
    values = result;
  });
  expect(values).toEqual(["\uD800", "\uDC00", "\uD800\uDC00", "\uFFFDd800", "漢😎"]);
});
