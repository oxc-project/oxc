import { describe, expect, it } from "vitest";
import { runCli } from "./utils";

describe("--stdin-filepath", () => {
  it("should format TS code from stdin", async () => {
    const proc = runCli(process.cwd(), ["--stdin-filepath", "test.ts"]);
    proc.stdin.write("const   x:number=1");
    proc.stdin.end();

    const result = await proc;
    expect({
      exitCode: result.exitCode,
      stdout: result.stdout,
    }).toMatchSnapshot();
  });

  it("should format GraphQL code from stdin", async () => {
    const proc = runCli(process.cwd(), ["--stdin-filepath", "test.graphql"]);
    proc.stdin.write("{   user(id:1){name}}");
    proc.stdin.end();

    const result = await proc;
    expect({
      exitCode: result.exitCode,
      stdout: result.stdout,
    }).toMatchSnapshot();
  });

  it("should fail for unsupported file type", async () => {
    const proc = runCli(process.cwd(), ["--stdin-filepath", "test.rb"]);
    proc.stdin.write("puts 'hello'");
    proc.stdin.end();

    const result = await proc;
    expect({
      exitCode: result.exitCode,
      stdout: result.stdout,
    }).toMatchSnapshot();
  });
});
