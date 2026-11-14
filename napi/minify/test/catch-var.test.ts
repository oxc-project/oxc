import { runInNewContext } from "node:vm";
import { expect, it } from "vitest";

import { minifySync } from "../index";

it("keeps nested catch values separate when mangling a captured binding", () => {
  const result = minifySync(
    "test.js",
    `
    function values() {
      var error;
      let inner, outer;
      try { throw 10; } catch (error) {
        try { throw 20; } catch (error) {
          var error = 30;
          inner = (() => { const local = 40; return [error, local]; })();
        }
        outer = error;
      }
      return [inner, outer, error];
    }
    values();
  `,
    { compress: false, mangle: true },
  );
  expect(result.errors).toEqual([]);
  expect(runInNewContext(result.code)).toEqual([[30, 40], 10, undefined]);
});
