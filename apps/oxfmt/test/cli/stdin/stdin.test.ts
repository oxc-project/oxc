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

  // https://github.com/oxc-project/oxc/issues/17939
  it("should not report `WouldBlock` error on large file piped to wc", async () => {
    const largeFile = await readFile(join(fixturesDir, "parser.ts"), "utf-8");
    const result = await runCliStdin(largeFile, "parser.ts", "wc -l");

    expect({
      exitCode: result.exitCode,
      stderr: result.stderr,
    }).toMatchSnapshot();
  });
});
