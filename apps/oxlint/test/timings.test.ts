import { join as pathJoin } from "node:path";

import { execa } from "execa";
import { expect, it } from "vitest";

import { PACKAGE_ROOT_PATH } from "./utils.ts";

const CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, "dist/cli.js");

async function runTimings(fixtureName: string): Promise<string> {
  const fixturePath = pathJoin(PACKAGE_ROOT_PATH, "test/fixtures", fixtureName);

  const { stdout } = await execa(
    process.execPath,
    [
      CLI_PATH,
      "--type-aware",
      "--debug",
      "timings",
      "--format",
      "default",
      "--threads",
      "1",
      "files",
    ],
    { cwd: fixturePath, reject: false },
  );
  return stdout;
}

it("reports per-rule timings for JS plugins", async () => {
  const stdout = await runTimings("basic_multiple_rules");

  expect(stdout).toMatch(
    /basic-custom-plugin\/no-debugger\s+\d+\.\d{3}\s+\d+\.\d%\s+2\s+js-plugin/,
  );
  expect(stdout).toMatch(
    /basic-custom-plugin\/no-debugger-2\s+\d+\.\d{3}\s+\d+\.\d%\s+2\s+js-plugin/,
  );
  expect(stdout).toMatch(
    /basic-custom-plugin\/no-identifiers-named-foo\s+\d+\.\d{3}\s+\d+\.\d%\s+2\s+js-plugin/,
  );
  expect(stdout).toMatch(
    /JS plugin runtime:\n  Total:\s+\d+\.\d{3}ms\n  Rule callbacks:\s+\d+\.\d{3}ms\n  Shared overhead:\s+\d+\.\d{3}ms/,
  );
});

it("reports JS plugin timings when rules produce no diagnostics", async () => {
  const stdout = await runTimings("basic_no_errors");

  expect(stdout).toMatch(/Found 0 warnings and 0 errors/);
  expect(stdout).toMatch(
    /basic-custom-plugin\/no-debugger\s+\d+\.\d{3}\s+\d+\.\d%\s+1\s+js-plugin/,
  );
});

it("reports JS plugin timings when a rule throws", async () => {
  const stdout = await runTimings("createOnce_hook_errors");

  expect(stdout).toContain("Error running JS plugin");
  expect(stdout).toMatch(
    /create-once-errors-plugin\/throw-in-before\s+\d+\.\d{3}\s+\d+\.\d%\s+\d+\s+js-plugin/,
  );
  expect(stdout).toMatch(
    /create-once-errors-plugin\/throw-in-visit\s+\d+\.\d{3}\s+\d+\.\d%\s+\d+\s+js-plugin/,
  );
  expect(stdout).toMatch(
    /create-once-errors-plugin\/throw-in-after\s+\d+\.\d{3}\s+\d+\.\d%\s+\d+\s+js-plugin/,
  );
});
