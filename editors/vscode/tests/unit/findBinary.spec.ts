import { strictEqual } from "assert";
import * as path from "node:path";
import { searchGlobalNodeModulesBin, searchProjectNodeModulesBin } from "../../client/findBinary";

suite("findBinary", () => {
  const binaryName = "oxlint";

  suite("searchProjectNodeModulesBin", () => {
    test("should return undefined when binary is not found in project node_modules", async () => {
      const result = await searchProjectNodeModulesBin("non-existent-binary-package-name-12345");
      strictEqual(result, undefined);
    });

    // this depends on the binary being installed in the oxc project's node_modules
    test("should replace dist/index.js with bin/<binary-name> in resolved path", async () => {
      const result = (await searchProjectNodeModulesBin(binaryName))!;

      strictEqual(result.includes(`${path.sep}dist${path.sep}index.js`), false);
      strictEqual(result.includes(`${path.sep}bin${path.sep}${binaryName}`), true);
    });
  });

  suite("searchGlobalNodeModulesBin", () => {
    test("should return undefined when binary is not found in global node_modules", async () => {
      const result = await searchGlobalNodeModulesBin("non-existent-binary-package-name-12345");
      strictEqual(result, undefined);
    });

    // Skipping this test as it may depend on the actual global installation of the binary
    test.skip("should replace dist/index.js with bin/<binary-name> in resolved path", async () => {
      const result = (await searchGlobalNodeModulesBin(binaryName))!;

      strictEqual(result.includes(`${path.sep}dist${path.sep}index.js`), false);
      strictEqual(result.includes(`${path.sep}bin${path.sep}${binaryName}`), true);
    });
  });
});
