import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Sort package.json fields", () => {
  it("should work with `format()` API", async () => {
    const pkgJSON = JSON.stringify({
      version: "1.0.0",
      name: "my-package",
    });
    const result1 = await format("package.json", pkgJSON);
    expect(result1.code).toBe(`{\n  "name": "my-package",\n  "version": "1.0.0"\n}\n`);
    expect(result1.errors).toStrictEqual([]);

    const result2 = await format("package.json", pkgJSON, { experimentalSortPackageJson: false });
    expect(result2.code).toBe(`{\n  "version": "1.0.0",\n  "name": "my-package"\n}\n`);
    expect(result2.errors).toStrictEqual([]);
  });
});
