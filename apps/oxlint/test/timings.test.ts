import { join as pathJoin } from "node:path";

import { execa } from "execa";
import { expect, it } from "vitest";

import { PACKAGE_ROOT_PATH } from "./utils.ts";

const CLI_PATH = pathJoin(PACKAGE_ROOT_PATH, "dist/cli.js");

async function runTimings(fixtureName: string, args: string[] = []): Promise<string> {
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
      ...args,
      "files",
    ],
    { cwd: fixturePath, reject: false },
  );
  return stdout;
}

it.each(["basic_multiple_rules", "basic_no_errors", "createOnce_hook_errors"])(
  "snapshots normalized JS plugin timings for %s",
  async (fixtureName) => {
    const stdout = await runTimings(fixtureName, ["--warn", "eslint/no-unused-vars"]);
    if (fixtureName === "basic_no_errors") {
      expect(stdout).toContain("Found 0 warnings and 0 errors");
    } else if (fixtureName === "createOnce_hook_errors") {
      expect(stdout).toContain("Error running JS plugin");
    }

    const timingsStart = stdout.indexOf("Rule timings:\n");
    expect(timingsStart).toBeGreaterThanOrEqual(0);

    // Diagnostics have their own fixture snapshots. Snapshot the complete timing output here.
    const lines = stdout.slice(timingsStart).trim().split("\n");
    const tableEnd = lines.indexOf("");
    expect(tableEnd).toBeGreaterThan(3);

    // Rules are ordered by measured duration, so sort by name before snapshotting.
    const normalized = [
      ...lines.slice(0, 3),
      ...lines.slice(3, tableEnd).sort(),
      ...lines.slice(tableEnd),
    ]
      .map((line) =>
        line
          // Preserve the width of each value so the original columns stay aligned.
          .replace(/\d+\.\d{3}/g, (value) => "0.000".padStart(value.length))
          .replace(/\d+\.\d%/g, (value) => "0.0%".padStart(value.length)),
      )
      .join("\n");

    expect(normalized).toMatchSnapshot();
  },
);
