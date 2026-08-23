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
    expect("y".replace(regexp, "$<constructor>")).toBe("");
    expect("y".replace(regexp, "$<hasOwnProperty>")).toBe("");
    expect("y".replace(regexp, "$<__proto__>")).toBe("");
    expect("y".replace(regexp, "$<>")).toBe("");
    expect("y".replace(regexp, "$$<missing>")).toBe("$<missing>");
    expect("y".replace(regexp, "$<missing")).toBe("$<missing");
  });

  test("preserves standard replacement tokens", () => {
    const native = /(x)|(y)/;
    const regexp = wrapRegExp(native, { a: [1, 2] });

    for (const replacement of [
      "$$",
      "$&",
      "$`",
      "$'",
      "$0",
      "$00",
      "$01",
      "$1",
      "$2",
      "$10",
      "$99",
      "$100",
    ]) {
      expect("ayb".replace(regexp, replacement)).toBe("ayb".replace(native, replacement));
    }
  });

  test("supports __proto__ as a named group", () => {
    const regexp = wrapRegExp(/(x)|(y)/, { ["__proto__"]: [1, 2] });

    expect("y".replace(regexp, "$<__proto__>")).toBe("y");
  });

  test("keeps following digits separate from capture indices", () => {
    const regexp = wrapRegExp(/(x)()()()()()()()()()()()/, { a: 1 });

    expect("x".replace(regexp, "$<a>2")).toBe("x2");
  });

  test("supports named captures above numeric replacement limits", () => {
    const regexp = wrapRegExp(new RegExp(`${"()".repeat(99)}(x)`), { a: 100 });

    expect("x".replace(regexp, "[$<a>]")).toBe("[x]");
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

  test("preserves named groups in matchAll clones", () => {
    const regexp = wrapRegExp(/(a)|(b)/g, { x: [1, 2] });
    const matches = [..."ab".matchAll(regexp)];

    expect(matches.map((match) => match.groups?.x)).toEqual(["a", "b"]);
  });
});
