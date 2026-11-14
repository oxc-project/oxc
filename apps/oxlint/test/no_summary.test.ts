import { join as pathJoin } from "node:path";

import { execa } from "execa";
import { expect, it } from "vitest";

import { PACKAGE_ROOT_PATH } from "./utils.ts";

const CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, "dist/cli.js");
const FIXTURE_PATH = pathJoin(PACKAGE_ROOT_PATH, "fixtures/cli/output_formatter_diagnostic");

function run(args: string[]) {
  return execa(process.execPath, [CLI_PATH, "--threads", "1", ...args], {
    cwd: FIXTURE_PATH,
    reject: false,
    env: { NO_COLOR: "1" },
  });
}

it.each(["default", "github", "stylish"])(
  "--no-summary preserves diagnostics and exit codes with %s output",
  async (format) => {
    const args = ["--format", format, "--no-summary", "-A", "all"];

    const clean = await run([...args, "ok.js"]);
    expect(clean.stdout).toBe("");
    expect(clean.exitCode).toBe(0);

    const warning = await run([...args, "-W", "no-debugger", "test.js"]);
    expect(warning.stdout).toContain("no-debugger");
    expect(warning.stdout).not.toMatch(/Found \d|Finished in|✖/);
    expect(warning.exitCode).toBe(0);

    const error = await run([...args, "-D", "no-debugger", "test.js"]);
    expect(error.stdout).toContain("no-debugger");
    expect(error.stdout).not.toMatch(/Found \d|Finished in|✖/);
    expect(error.exitCode).toBe(1);

    const denied = await run([...args, "-W", "no-debugger", "--deny-warnings", "test.js"]);
    expect(denied.stdout).toContain("no-debugger");
    expect(denied.exitCode).toBe(1);

    const quiet = await run([...args, "-W", "no-debugger", "--quiet", "test.js"]);
    expect(quiet.stdout).toBe("");
    expect(quiet.exitCode).toBe(0);
  },
);

it("--no-summary preserves the warning limit message and no-files errors", async () => {
  const args = ["--format", "default", "--no-summary"];
  const warnings = await run([
    ...args,
    "-A",
    "all",
    "-W",
    "no-debugger",
    "--quiet",
    "--max-warnings",
    "0",
    "test.js",
  ]);
  expect(warnings.stdout).toBe("Exceeded maximum number of warnings. Found 1.");
  expect(warnings.exitCode).toBe(1);

  const missing = await run([...args, "missing.js"]);
  expect(missing.stdout).toContain("No files found to lint.");
  expect(missing.stdout).not.toContain("Finished in");
  expect(missing.exitCode).toBe(1);

  const allowed = await run([...args, "--no-error-on-unmatched-pattern", "missing.js"]);
  expect(allowed.stdout).toBe("");
  expect(allowed.exitCode).toBe(0);
});

it.each(["agent", "checkstyle", "gitlab", "junit", "sarif", "unix"])(
  "--no-summary preserves %s output",
  async (format) => {
    const args = ["--format", format, "-A", "all", "-D", "no-debugger", "test.js"];
    const original = await run(args);
    const withoutSummary = await run(["--no-summary", ...args]);
    expect(withoutSummary.stdout).toBe(original.stdout);
    expect(withoutSummary.exitCode).toBe(original.exitCode);
  },
);

it("--no-summary preserves JSON diagnostics and metadata", async () => {
  const args = ["--format", "json", "-A", "all", "-D", "no-debugger", "test.js"];
  const original = await run(args);
  const withoutSummary = await run(["--no-summary", ...args]);
  expect(JSON.parse(withoutSummary.stdout)).toEqual({
    ...JSON.parse(original.stdout),
    start_time: expect.any(Number),
  });
  expect(withoutSummary.exitCode).toBe(original.exitCode);
});
