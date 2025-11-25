import { describe, it, expect } from "vitest";
import { clearLoadedPlugin } from "../dist/index.js";

/**
 * Test the clearLoadedPlugin function
 */
describe("clearLoadedPlugin", () => {
  it("should be exported from the main module", () => {
    expect(clearLoadedPlugin).toBeDefined();
    expect(typeof clearLoadedPlugin).toBe("function");
  });

  it("should execute without errors", async () => {
    // Call the clear function - it should not throw
    await expect(clearLoadedPlugin()).resolves.toBeUndefined();
  });

  it("should work when called multiple times", async () => {
    // Call it multiple times to ensure it doesn't error
    await expect(clearLoadedPlugin()).resolves.toBeUndefined();
    await expect(clearLoadedPlugin()).resolves.toBeUndefined();
    await expect(clearLoadedPlugin()).resolves.toBeUndefined();
  });
});
