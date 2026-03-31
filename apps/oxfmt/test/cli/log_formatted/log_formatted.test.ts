import { describe, expect, it } from "vitest";
import { join } from "node:path";
import fs from "node:fs/promises";
import { tmpdir } from "node:os";
import { runCli } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

async function runInTempDir(files: string[], args: string[]) {
  const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-test-"));
  try {
    await fs.cp(fixturesDir, tempDir, { recursive: true });
    const result = await runCli(tempDir, [...args, ...files]);
    return {
      stdout: result.stdout
        .replace(/\d+(?:\.\d+)?s|\d+ms/g, "<variable>ms")
        .replace(/\\/g, "/"),
      stderr: result.stderr,
      exitCode: result.exitCode,
    };
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

describe("log_formatted", () => {
  it("should log changed file paths with --log-formatted", async () => {
    const result = await runInTempDir(
      ["unformatted.js"],
      ["--log-formatted"],
    );
    expect(result.stdout).toMatchSnapshot();
    expect(result.exitCode).toBe(0);
  });

  it("should not log paths when file is already formatted", async () => {
    const result = await runInTempDir(
      ["already_formatted.js"],
      ["--log-formatted"],
    );
    expect(result.stdout).toMatchSnapshot();
    expect(result.exitCode).toBe(0);
  });

  it("should not log paths without --log-formatted flag", async () => {
    const result = await runInTempDir(["unformatted.js"], []);
    expect(result.stdout).toMatchSnapshot();
    expect(result.exitCode).toBe(0);
  });

  it("should log only changed files when mixed with already formatted", async () => {
    const result = await runInTempDir(
      ["unformatted.js", "already_formatted.js"],
      ["--log-formatted"],
    );
    expect(result.stdout).toMatchSnapshot();
    expect(result.exitCode).toBe(0);
  });
});
