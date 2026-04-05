import { spawnSync } from "node:child_process";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const TEST_DIR_PATH = dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT_PATH = dirname(TEST_DIR_PATH);

function runHelp(scriptRelativePath: string) {
  return spawnSync(
    process.execPath,
    ["--experimental-strip-types", scriptRelativePath, "--", "--help"],
    {
      cwd: PACKAGE_ROOT_PATH,
      encoding: "utf8",
    },
  );
}

describe("oxlint real-package Svelte script contracts", () => {
  it("accepts pnpm-style -- separators before --help", () => {
    for (const scriptRelativePath of [
      "./scripts/run-real-svelte-package-tests.ts",
      "./scripts/install-real-svelte-packages.ts",
      "./scripts/check-real-svelte-packages.ts",
      "./scripts/report-real-svelte-packages.ts",
      "./scripts/annotate-real-svelte-packages.ts",
      "./scripts/cleanup-real-svelte-packages.ts",
    ]) {
      const result = runHelp(scriptRelativePath);
      expect(result.status, `expected ${scriptRelativePath} to exit cleanly`).toBe(0);
      expect(result.stdout).toContain("Usage:");
      expect(result.stderr).toBe("");
    }
  });
});
