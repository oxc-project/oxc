import { expect, test } from "vitest";

import { minifySync } from "../index";

const sources = [
  String.raw`capture(["a".concat("\uD800", "", "\uDC00"), "\uD800".concat("x", "\uDC00")])`,
  String.raw`capture(["\uD800".length, "\uD800".charCodeAt(0), "\uD800"[0], "𐀀"[0], "𐀀"[1]])`,
  String.raw`capture([+"\uD800", +"\uFFFDd800", "\uD800" === "\uFFFDd800", "\uD800" === "\uD800"])`,
  String.raw`capture(["\uD800" + "\uDC00", "\uD800" + "x", ["\uD800", "\uDC00", "\uFFFDd800"].join("")])`,
  'capture([`\\uD800${"\\uDC00"}`, `${"\\uD800"}\\uDC00`, `\\0${"1"}`, `$${"{"}`])',
  String.raw`capture(["\uD800\0" + "1", "\uD800\n\"\\", "\uD800</script>", "\uD800\${x}"])`,
  String.raw`capture(["\uD800", "\uDC00", "\uD801", "\uDC01", "\uD802", "\uDC02", "\uD803", "\uDC03"])`,
];

function evaluate(source: string): unknown {
  let result: unknown;
  new Function("capture", source)((value: unknown) => {
    result = value;
  });
  return result;
}

test.each(sources)("preserves JavaScript string behavior: %s", (source) => {
  const result = minifySync("test.js", source);
  expect(result.errors).toEqual([]);
  expect(evaluate(result.code)).toEqual(evaluate(source));
  const repeated = minifySync("test.js", result.code);
  expect(repeated.errors).toEqual([]);
  expect(evaluate(repeated.code)).toEqual(evaluate(source));
});
