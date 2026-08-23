import { createRequire } from "node:module";

import { describe, expect, test } from "vitest";

import wrapRegExpEsm from "../../../npm/runtime/src/helpers/esm/wrapRegExp.js";

type WrapRegExp = (regexp: RegExp, groups: Record<string, number | number[]>) => RegExp;

const require = createRequire(import.meta.url);
const wrapRegExpCjs = require("../../../npm/runtime/src/helpers/wrapRegExp.js") as WrapRegExp;

describe.each([
  ["CommonJS", wrapRegExpCjs],
  ["ESM", wrapRegExpEsm],
] as const)("wrapRegExp %s helper", (_, wrapRegExp) => {
  test("respects escaped dollars in named replacements", () => {
    const regexp = wrapRegExp(/(x)|(y)/, { a: [1, 2] });

    expect("y".replace(regexp, "$$<a>")).toBe("$<a>");
    expect("y".replace(regexp, "$$$<a>")).toBe("$y");
    expect("y".replace(regexp, "$$$$<a>")).toBe("$$<a>");
    expect("y".replace(regexp, "$<a>")).toBe("y");
    expect("y".replace(regexp, "$<missing>")).toBe("");
    expect("y".replace(regexp, "$$<missing>")).toBe("$<missing>");
    expect("y".replace(regexp, "$<missing")).toBe("$<missing");
  });

  test("keeps following digits separate from capture indices", () => {
    const regexp = wrapRegExp(/(x)()()()()()()()()()()()/, { a: 1 });

    expect("x".replace(regexp, "$<a>2")).toBe("x2");
  });

  test("supports ordinary construction through a wrapped instance", () => {
    const regexp = wrapRegExp(/(x)|(y)/, { a: [1, 2] });
    const Constructor = regexp.constructor as RegExpConstructor;
    const ordinary = new Constructor("z");

    expect(ordinary.test("z")).toBe(true);
    expect("z".replace(ordinary, "$<a>")).toBe("$<a>");
    expect("z".replace(ordinary, () => "replaced")).toBe("replaced");
  });

  test("preserves identity through the callable RegExp constructor", () => {
    const regexp = wrapRegExp(/(x)|(y)/, { a: [1, 2] });

    expect(RegExp(regexp)).toBe(regexp);
  });
});
