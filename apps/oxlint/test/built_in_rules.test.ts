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
