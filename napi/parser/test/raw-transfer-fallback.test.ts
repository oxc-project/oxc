import { describe, expect, it, vi } from "vitest";

import { parseSync } from "../src-js/index.js";
// `index.js` loads raw transfer code with `require`, which `vi.mock` does not affect, so import it directly
// @ts-expect-error - `eager.js` has no type definitions
import { parseSyncRaw } from "../src-js/raw-transfer/eager.js";

// Make allocating raw transfer buffers on Rust side fail, so they're allocated on JS side instead.
// This is the path taken in runtimes which do not support external `ArrayBuffer`s (e.g. Electron).
const createRawTransferBuffer = vi.hoisted(() => vi.fn(() => null));
vi.mock("../src-js/bindings.js", async (importOriginal) => ({
  ...(await importOriginal<Record<string, unknown>>()),
  createRawTransferBuffer,
}));

describe("raw transfer", () => {
  it("falls back to allocating buffer on JS side", () => {
    const code = 'const s = "日本"; function f(a, b) { return a + b; }';
    const raw = parseSyncRaw("test.js", code, {});
    expect(JSON.stringify(raw.program)).toBe(JSON.stringify(parseSync("test.js", code).program));

    expect(createRawTransferBuffer).toHaveBeenCalled();
    // Buffer was allocated on JS side, as a 6 GiB `ArrayBuffer`
    expect(process.memoryUsage().arrayBuffers).toBeGreaterThanOrEqual(2 ** 32);
  });
});
