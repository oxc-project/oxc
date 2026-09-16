import { join as pathJoin } from "node:path";
import { describe, expect, it, vi } from "vitest";

const path = pathJoin(import.meta.dirname, "dummy.js");

describe("parse", () => {
  it("keeps working after the module registry is reset", async () => {
    // Rust keeps its state across a module reset, JS does not.
    // `RuleTester` must still get a usable buffer from the second module instance.
    const first = await import("../src-js/package/parse.ts");
    first.parse(path, "let a = 1;");

    vi.resetModules();

    const second = await import("../src-js/package/parse.ts");
    expect(second).not.toBe(first);
    const bufferId = second.parse(path, "let b = 2;");

    const { buffers } = await import("../src-js/plugins/lint.ts");
    expect(buffers[bufferId]).toBeDefined();
  });
});
