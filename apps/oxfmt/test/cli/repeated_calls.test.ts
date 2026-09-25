import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import Tinypool from "tinypool";
import { expect, it } from "vitest";
import { runCli } from "../../src-js/bindings";
import { loadJsConfig } from "../../src-js/cli/js_config";
import { toFormatFileResult, toNullable } from "../../src-js/libs/napi-callbacks";

it("formats files on repeated CLI calls in the same process", async () => {
  const directory = await mkdtemp(join(tmpdir(), "oxfmt-repeated-cli-"));
  const configPath = join(directory, "oxfmt.config.mjs");
  const sourcePath = join(directory, "input.js");
  const markdownPath = join(directory, "input.md");
  let pool: Tinypool | undefined;

  try {
    await writeFile(configPath, "export default { semi: false };\n");

    // Calls must be sequential to exercise reinitialization after worker cleanup.
    /* oxlint-disable no-await-in-loop */
    // Rewrite both files before each call so the second run must also format them.
    for (let call = 0; call < 2; call++) {
      await writeFile(sourcePath, "const   value=1;\n");
      await writeFile(markdownPath, "#   Heading\n");

      const result = await runCli(
        ["--threads=1", "--config", configPath, "--write", sourcePath, markdownPath],
        loadJsConfig,
        async (numThreads) => {
          pool = new Tinypool({
            filename: new URL("../../dist/cli-worker.js", import.meta.url).href,
            minThreads: numThreads,
            maxThreads: numThreads,
            runtime: "child_process",
            env: process.env as Record<string, string>,
          });
        },
        (options, code) => toFormatFileResult(pool!.run({ options, code }, { name: "formatFile" })),
        (options, code) => toNullable(pool!.run({ options, code }, { name: "formatEmbeddedCode" })),
        (options, code) => toNullable(pool!.run({ options, code }, { name: "formatEmbeddedDoc" })),
        (options, classes) =>
          toNullable(pool!.run({ options, classes }, { name: "sortTailwindClasses" })),
      );
      await pool?.destroy();
      pool = undefined;

      expect(result).toEqual(["cli", 0]);
      expect(await readFile(sourcePath, "utf8")).toBe("const value = 1\n");
      expect(await readFile(markdownPath, "utf8")).toBe("# Heading\n");
    }
    /* oxlint-enable no-await-in-loop */
  } finally {
    await pool?.destroy();
    await rm(directory, { recursive: true, force: true });
  }
});
