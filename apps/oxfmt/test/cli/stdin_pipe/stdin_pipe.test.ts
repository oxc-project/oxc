import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { execa } from "execa";

const CLI_PATH = join(import.meta.dirname, "..", "..", "..", "dist", "cli.js");
const FIXTURE = join(import.meta.dirname, "fixtures", "parser.ts");

// https://github.com/oxc-project/oxc/issues/17939
describe("stdin pipe", () => {
  it("should not report WouldBlock error on large file piped to wc", async () => {
    const cmd = `cat "${FIXTURE}" | node "${CLI_PATH}" --stdin-filepath=parser.ts | wc -l`;
    const result = await execa({ shell: true, reject: false })`${cmd}`;

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
  });
});
