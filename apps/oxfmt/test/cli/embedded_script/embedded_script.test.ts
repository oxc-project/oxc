import { describe, expect, test } from "vitest";
import { join } from "node:path";
import { runCli } from "../utils";

const fixturesDir = join(import.meta.dirname, "fixtures");

describe("embedded_script", () => {
  test("should sort imports in Vue <script> tag", async () => {
    const cwd = join(fixturesDir, "vue_sort_imports");
    const result = await runCli(cwd, ["--check", "input.vue"]);

    expect(result.exitCode).toBe(0);
  });

  test("should sort Tailwind classes in Vue <script> JSX", async () => {
    const cwd = join(fixturesDir, "vue_tailwind");
    const result = await runCli(cwd, ["--check", "input.vue"]);

    expect(result.exitCode).toBe(0);
  });

  test("should sort imports in HTML <script> tag", async () => {
    const cwd = join(fixturesDir, "html_sort_imports");
    const result = await runCli(cwd, ["--check", "input.html"]);

    expect(result.exitCode).toBe(0);
  });
});
