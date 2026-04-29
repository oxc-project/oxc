import { describe, expect, it } from "vitest";
import { allRules } from "../src-js/package/built-in-rules.ts";

const ALLOWED_CATEGORIES = new Set([
  "correctness",
  "suspicious",
  "pedantic",
  "perf",
  "style",
  "restriction",
  "nursery",
]);

describe("built-in rules metadata", () => {
  it("exports a non-empty rule list", () => {
    expect(allRules.length).toBeGreaterThan(0);
  });

  it("contains the requested metadata fields", () => {
    for (const rule of allRules) {
      expect(typeof rule.plugin).toBe("string");
      expect(typeof rule.rule).toBe("string");
      expect(ALLOWED_CATEGORIES.has(rule.category)).toBe(true);
    }
  });

  it("uses public config plugin names", () => {
    expect(allRules.some((rule) => rule.plugin === "jsx-a11y" && rule.rule === "alt-text")).toBe(
      true,
    );
    expect(
      allRules.some((rule) => rule.plugin === "react-perf" && rule.rule === "jsx-no-jsx-as-prop"),
    ).toBe(true);
    expect(allRules.some((rule) => (rule.plugin as string) === "jsx_a11y")).toBe(false);
    expect(allRules.some((rule) => (rule.plugin as string) === "react_perf")).toBe(false);
  });

  it("includes vitest aliases for jest-compatible rules", () => {
    expect(
      allRules.some(
        (rule) =>
          rule.plugin === "vitest" &&
          rule.rule === "expect-expect" &&
          rule.category === "correctness",
      ),
    ).toBe(true);
    expect(
      allRules.some(
        (rule) =>
          rule.plugin === "vitest" &&
          rule.rule === "no-restricted-vi-methods" &&
          rule.category === "style",
      ),
    ).toBe(true);
  });
});
