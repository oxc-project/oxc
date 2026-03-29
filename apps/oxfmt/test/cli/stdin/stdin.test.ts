import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { readFile } from "node:fs/promises";
import { runCliStdin } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("--stdin-filepath", () => {
  it("should format TS code from stdin", async () => {
    const result = await runCliStdin("const   x:number=1", "test.ts");
    expect({
      exitCode: result.exitCode,
      stdout: result.stdout,
    }).toMatchSnapshot();
  });

  it("should format GraphQL code from stdin", async () => {
    const result = await runCliStdin("{   user(id:1){name}}", "test.graphql");
    expect({
      exitCode: result.exitCode,
      stdout: result.stdout,
    }).toMatchSnapshot();
  });

  it("should fail for unsupported file type", async () => {
    const result = await runCliStdin("puts 'hello'", "test.rb");
    expect({
      exitCode: result.exitCode,
      stdout: result.stdout,
    }).toMatchSnapshot();
  });

  it("should respect ignorePatterns for stdin-filepath", async () => {
    const fixtureCwd = join(fixturesDir, "ignore-patterns");
    const input = "const   x:number=1";

    const ignored = await runCliStdin(input, "ignored/file.ts", {
      cwd: fixtureCwd,
      extraArgs: ["--config=.oxfmtrc.json"],
    });

    expect({
      exitCode: ignored.exitCode,
      stdout: ignored.stdout,
    }).toMatchSnapshot();

    expect(ignored.stdout).toBe(input);

    const formatted = await runCliStdin(input, "src/file.ts", {
      cwd: fixtureCwd,
      extraArgs: ["--config=.oxfmtrc.json"],
    });
    expect({
      exitCode: formatted.exitCode,
      stdout: formatted.stdout,
    }).toMatchSnapshot();

    expect(formatted.stdout).not.toBe(input);
  });

  // https://github.com/oxc-project/oxc/issues/17939
  it("should not report `WouldBlock` error on large file piped to wc", async () => {
    const largeFile = await readFile(join(fixturesDir, "parser.ts"), "utf-8");
    const result = await runCliStdin(largeFile, "parser.ts", { pipe: "wc -l" });

    expect({
      exitCode: result.exitCode,
      stderr: result.stderr,
    }).toMatchSnapshot();
  });
});
