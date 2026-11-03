// oxlint-disable typescript/unbound-method -- Getter receiver behavior is the contract under test.
import { describe, expect, it } from "vitest";
import { rawTransferSupported } from "../src-js/index.js";
import { parse, parseSync } from "./parser.ts";

const rawOptions = { experimentalRawTransfer: true };

describe.skipIf(!rawTransferSupported())("eager raw result storage", () => {
  it.each([false, true])(
    "preserves own accessor descriptors with tokens=%s",
    (experimentalTokens) => {
      const result = parseSync("test.js", "/* comment */ export const x = 1;", {
        ...rawOptions,
        experimentalTokens,
      });
      const keys = [
        "program",
        "module",
        "comments",
        ...(experimentalTokens ? ["tokens"] : []),
        "errors",
      ];
      expect(Reflect.ownKeys(result)).toEqual(keys);
      expect(Object.getPrototypeOf(result)).toBe(Object.prototype);
      expect(Object.prototype.toString.call(result)).toBe("[object Object]");
      for (const key of keys) {
        const descriptor = Object.getOwnPropertyDescriptor(result, key)!;
        expect(descriptor).toEqual({
          get: expect.any(Function),
          set: undefined,
          enumerable: true,
          configurable: true,
        });
        expect(descriptor.get!.name).toBe(`get ${key}`);
        expect(descriptor.get!.length).toBe(0);
        expect(() => {
          (result as any)[key] = null;
        }).toThrow(TypeError);
      }
      expect(Object.keys({ ...(result as object) })).toEqual(keys);
      expect(Object.keys(structuredClone(result))).toEqual(keys);
      Object.freeze(result);
      expect(result.program.body).toHaveLength(1);
    },
  );

  it("shares getters and requires the branded receiver", () => {
    const first = parseSync("first.js", "let first;", rawOptions);
    const second = parseSync("second.js", "let second;", rawOptions);
    for (const key of ["program", "module", "comments", "errors"] as const) {
      const getter = Object.getOwnPropertyDescriptor(first, key)!.get!;
      expect(getter).toBe(Object.getOwnPropertyDescriptor(second, key)!.get);
      expect(getter.call(first)).toBe(first[key]);
      expect(getter.call(second)).toBe(second[key]);
      expect(() => getter.call({})).toThrow(TypeError);
      expect(() => Reflect.get(first, key, {})).toThrow(TypeError);
      expect(() => new Proxy(first, {})[key]).toThrow(TypeError);
      expect(() => Object.create(first)[key]).toThrow(TypeError);
      expect(
        new Proxy(first, {
          get(target, property) {
            return Reflect.get(target, property, target);
          },
        })[key],
      ).toBe(first[key]);
    }
  });

  it("keeps results independent across buffer reuse and property redefinition", () => {
    const first = parseSync("first.js", "/* first */ let first;", rawOptions);
    const { program, comments } = first;
    for (let i = 0; i < 64; i++) parseSync("other.ts", `type T = ${i};`, rawOptions);
    expect(first.program).toBe(program);
    expect(first.comments).toBe(comments);
    const getter = Object.getOwnPropertyDescriptor(first, "program")!.get!;
    Object.defineProperty(first, "program", { value: null });
    expect(first.program).toBe(null);
    expect(getter.call(first)).toBe(program);
    expect(delete (first as any).comments).toBe(true);
    expect(first).not.toHaveProperty("comments");
  });

  it.each([
    ["test.js", "#!/usr/bin/env node\r\n/* 雪😀 */ const x = ['\\uD800', 123n, /雪/gu];"],
    ["test.ts", "#!/usr/bin/env node\n/* 雪😀 */ type T = string; const x: T = '\\uDFFF';"],
    ["test.jsx", "// 😀\r\nexport default <div title='雪'>😀</div>;"],
    ["test.tsx", "/* 😀 */ export const x: JSX.Element = <div>{123n}</div>;"],
    ["test.js", "export const = ;"],
    ["test.ts", "let x: = ;"],
  ])("matches JSON for %s: %s", (filename, source) => {
    for (const range of [false, true]) {
      const expected = parseSync(filename, source, { range });
      const actual = parseSync(filename, source, { ...rawOptions, range });
      for (const key of ["program", "module", "comments", "errors"] as const) {
        const value =
          range && key !== "program"
            ? JSON.parse(
                JSON.stringify(actual[key], (name, entry) =>
                  name === "range" ? undefined : entry,
                ),
              )
            : actual[key];
        expect(value).toEqual(expected[key]);
      }
    }
  });

  it("exposes deserialized tokens", () => {
    const result = parseSync("test.js", "/* comment */ export const x = 1;", {
      ...rawOptions,
      experimentalTokens: true,
    });
    expect((result as any).tokens.map((token: any) => token.value)).toEqual([
      "export",
      "const",
      "x",
      "=",
      "1",
      ";",
    ]);
  });

  it("supports parent links and tokens together", () => {
    const result = parseSync("test.js", "/* 😀 */ let x = 123n;", {
      ...rawOptions,
      range: true,
      experimentalParent: true,
      experimentalTokens: true,
    });
    expect(result.program.parent).toBe(null);
    expect(result.program.body[0].parent).toBe(result.program);
    expect((result as any).tokens[0]).toMatchObject({ value: "let", start: 9, end: 12 });
  });

  it("supports queued async parses and syntax diagnostics", async () => {
    const inputs = Array.from({ length: 32 }, (_, i) => `/* ${i} */ const x = ${i};`);
    inputs.push("const = ;");
    const results = await Promise.all(inputs.map((source) => parse("test.js", source, rawOptions)));
    for (let i = 0; i < inputs.length; i++) {
      const expected = parseSync("test.js", inputs[i]);
      for (const key of ["program", "module", "comments", "errors"] as const) {
        expect(results[i][key]).toEqual(expected[key]);
      }
    }
    await expect(parse("test.js", null as any, rawOptions)).rejects.toThrow(TypeError);
    expect(() => parseSync("test.js", null as any, rawOptions)).toThrow(TypeError);
  });
});
