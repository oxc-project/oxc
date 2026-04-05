import { describe, expect, it } from "vitest";
import { join } from "node:path";
import { readFile, rm, writeFile } from "node:fs/promises";
import { runCli, runCliStdin, runWriteModeAndSnapshot } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");
const expectedOutput = `<script>
  export let name = "world";
</script>

<h1>Hello {name}</h1>

<style>
  h1 {
    color: red;
  }
</style>
`;

describe("plugin_languages_real_package", () => {
  it("should format .svelte through the real prettier-plugin-svelte package from local node_modules", async () => {
    const snapshot = await runWriteModeAndSnapshot(fixturesDir, ["App.svelte"]);

    expect(snapshot).toContain(`--- FILE -----------
App.svelte`);
    expect(snapshot).toContain(
      `--- BEFORE ---------
<style>h1{color:red}</style>
<h1>Hello {name}</h1>
<script>export let name = "world";</script>`,
    );
    expect(snapshot).toContain(`--- AFTER ----------
${expectedOutput}`.trimEnd());
  });

  it("should format stdin .svelte through the real prettier-plugin-svelte package from local node_modules", async () => {
    const input = await readFile(join(fixturesDir, "App.svelte"), "utf-8");
    const result = await runCliStdin(input, "App.svelte", { cwd: fixturesDir });

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(result.stdout).toBe(expectedOutput);
  });

  it("should discover override-scoped real prettier-plugin-svelte config for stdin from the config directory", async () => {
    const input = await readFile(join(fixturesDir, "subdir", "config", "App.svelte"), "utf-8");
    const result = await runCliStdin(input, "subdir/config/App.svelte", {
      cwd: fixturesDir,
      args: ["-c", "./subdir/config/.oxfmtrc.json"],
    });

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(result.stdout).toBe(expectedOutput);
  });

  it("should discover override-scoped real prettier-plugin-svelte config from the config directory", async () => {
    const snapshot = await runWriteModeAndSnapshot(
      fixturesDir,
      ["subdir/config/App.svelte"],
      ["-c", "./subdir/config/.oxfmtrc.json"],
    );

    expect(snapshot).toContain(`--- FILE -----------
subdir/config/App.svelte`);
    expect(snapshot).toContain(
      `--- BEFORE ---------
<style>h1{color:red}</style>
<h1>Hello {name}</h1>
<script>export let name = "world";</script>`,
    );
    expect(snapshot).toContain(`--- AFTER ----------
${expectedOutput}`.trimEnd());
  });

  it("should treat a newline-only EOF change from the real plugin as clean in --check", async () => {
    const result = await runCli(fixturesDir, ["--check", "AlreadyFormattedNoFinalNewline.svelte"]);

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(result.stdout).toContain("All matched files use the correct format.");
  });

  it("should omit a newline-only EOF change from the real plugin in --list-different", async () => {
    const result = await runCli(fixturesDir, ["--list-different", "AlreadyFormattedNoFinalNewline.svelte"]);

    expect(result.exitCode).toBe(0);
    expect(result.stderr).toBe("");
    expect(result.stdout).toBe("");
  });

  it("should treat an empty newline-only EOF change from the real plugin as clean in --check", async () => {
    const tempFile = join(fixturesDir, "__temp_empty_no_final_newline__.svelte");

    try {
      await writeFile(tempFile, "");

      const result = await runCli(fixturesDir, ["--check", "__temp_empty_no_final_newline__.svelte"]);

      expect(result.exitCode).toBe(0);
      expect(result.stderr).toBe("");
      expect(result.stdout).toContain("All matched files use the correct format.");
    } finally {
      await rm(tempFile, { force: true });
    }
  });

  it("should omit an empty newline-only EOF change from the real plugin in --list-different", async () => {
    const tempFile = join(fixturesDir, "__temp_empty_no_final_newline__.svelte");

    try {
      await writeFile(tempFile, "");

      const result = await runCli(fixturesDir, [
        "--list-different",
        "__temp_empty_no_final_newline__.svelte",
      ]);

      expect(result.exitCode).toBe(0);
      expect(result.stderr).toBe("");
      expect(result.stdout).toBe("");
    } finally {
      await rm(tempFile, { force: true });
    }
  });
});
