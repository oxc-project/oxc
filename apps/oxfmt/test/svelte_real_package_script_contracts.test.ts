import { spawnSync } from "node:child_process";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  collectAnnotations,
  emitAnnotation,
} from "../scripts/annotate-real-svelte-packages.ts";
import { renderMarkdownReport } from "../scripts/report-real-svelte-packages.ts";

const TEST_DIR_PATH = dirname(fileURLToPath(import.meta.url));
const PACKAGE_ROOT_PATH = dirname(TEST_DIR_PATH);
const previousGithubActions = process.env.GITHUB_ACTIONS;

afterEach(() => {
  vi.restoreAllMocks();

  if (previousGithubActions === undefined) {
    delete process.env.GITHUB_ACTIONS;
  } else {
    process.env.GITHUB_ACTIONS = previousGithubActions;
  }
});

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

describe("oxfmt real-package Svelte diagnostics scripts", () => {
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

  it("renders the markdown report with package, link, resolution, and failed-step sections", () => {
    const markdown = renderMarkdownReport({
      generatedAt: "2026-04-04T00:00:00.000Z",
      profileName: "pinned",
      platform: "linux",
      nodeVersion: "v22.0.0",
      packageRootPath: "/repo/apps/oxfmt",
      installRootPath: "/repo/apps/oxfmt/test/.real-svelte-packages",
      installNodeModulesPath: "/repo/apps/oxfmt/test/.real-svelte-packages/node_modules",
      installPackageJsonPath: "/repo/apps/oxfmt/test/.real-svelte-packages/package.json",
      installPackageJsonMatches: false,
      managedRunState: {
        build: false,
        createdAt: "2026-04-04T00:00:00.000Z",
        failureMessage: "smoke failed",
        keepOnFailure: false,
        laneStatus: "failed",
        profileName: "pinned",
        reportMode: "failure",
        requestedSuites: ["api", "smoke"],
        steps: [
          {
            stepName: "smoke",
            scriptName: "test:svelte-real-packages:smoke",
            extraArgs: [],
            startedAt: "2026-04-04T00:00:00.000Z",
            finishedAt: "2026-04-04T00:01:00.000Z",
            status: "failed",
            errorMessage: "Command failed: pnpm run test:svelte-real-packages:smoke (exit code 1).",
            exitCode: 1,
          },
        ],
        updatedAt: "2026-04-04T00:01:00.000Z",
      },
      managedRunStatePath: "/repo/apps/oxfmt/test/.real-svelte-packages-state.json",
      installPackageJsonDependencies: {
        prettier: "3.8.1",
        svelte: "5.55.1",
        "prettier-plugin-svelte": "3.5.1",
      },
      requestedDependencySpecifiers: {
        prettier: "3.8.1",
        svelte: "5.55.1",
        "prettier-plugin-svelte": "3.5.1",
      },
      packages: [
        {
          packageName: "prettier-plugin-svelte",
          requestedSpecifier: "3.5.1",
          expectedVersion: "3.5.1",
          installedVersion: "3.5.0",
          status: "version-mismatch",
          packageJsonPath: "/repo/apps/oxfmt/test/.real-svelte-packages/node_modules/prettier-plugin-svelte/package.json",
        },
      ],
      links: [
        {
          label: "runtime fixture node_modules",
          linkPath: "/repo/apps/oxfmt/test/fixtures/node_modules",
          exists: true,
          isSymlink: true,
          resolvedPath: "/wrong/target",
          expectedTargetPath: "/repo/apps/oxfmt/test/.real-svelte-packages/node_modules",
          status: "wrong-target",
          error: null,
        },
      ],
      resolutions: [
        {
          scopeLabel: "runtime fixture",
          fromDirPath: "/repo/apps/oxfmt/test/fixtures",
          packageName: "prettier-plugin-svelte",
          resolvedPath: "/repo/apps/oxfmt/test/fixtures/node_modules/prettier-plugin-svelte/index.js",
          realPath: "/wrong/target/prettier-plugin-svelte/index.js",
          insideInstallNodeModules: false,
          error: null,
        },
      ],
      npmLs: {
        exitCode: 1,
        stdout: "",
        stderr: "npm ls found problems",
      },
    });

    expect(markdown).toContain("# Oxfmt real-package Svelte report (pinned)");
    expect(markdown).toContain("Install package.json matches requested dependencies: no");
    expect(markdown).toContain("prettier-plugin-svelte: version-mismatch");
    expect(markdown).toContain("runtime fixture node_modules: wrong-target");
    expect(markdown).toContain("[runtime fixture] prettier-plugin-svelte: ok");
    expect(markdown).toContain("## Failed managed steps");
    expect(markdown).toContain("smoke: Command failed: pnpm run test:svelte-real-packages:smoke (exit code 1).");
    expect(markdown).toContain("npm ls found problems");
  });

  it("collects annotations for missing reports and for concrete report failures", () => {
    const missingReportAnnotations = collectAnnotations(
      null,
      "pinned",
      "/repo/apps/oxfmt/test/.real-svelte-packages-report.json",
      "/repo/apps/oxfmt/test/.real-svelte-packages-state.json",
    );

    expect(missingReportAnnotations).toHaveLength(1);
    expect(missingReportAnnotations[0]).toMatchObject({
      level: "warning",
      title: "Oxfmt Svelte diagnostics report missing",
    });

    const annotations = collectAnnotations(
      {
        installPackageJsonMatches: false,
        managedRunState: {
          laneStatus: "failed",
          failureMessage: "lane failed",
          steps: [
            {
              stepName: "smoke",
              scriptName: "test:svelte-real-packages:smoke",
              status: "failed",
              exitCode: 1,
              errorMessage: "smoke failed",
            },
          ],
        },
        npmLs: {
          exitCode: 1,
          stderr: "npm ls failed",
        },
        packages: [
          {
            packageName: "prettier-plugin-svelte",
            status: "missing",
          },
        ],
        profileName: "pinned",
        links: [
          {
            label: "runtime fixture node_modules",
            linkPath: "/repo/apps/oxfmt/test/fixtures/node_modules",
            status: "wrong-target",
            error: null,
          },
        ],
        resolutions: [
          {
            packageName: "prettier-plugin-svelte",
            scopeLabel: "runtime fixture",
            insideInstallNodeModules: false,
            realPath: "/wrong/target/prettier-plugin-svelte/index.js",
            error: null,
          },
          {
            packageName: "svelte",
            scopeLabel: "runtime fixture",
            error: "Cannot find module 'svelte'",
          },
        ],
      },
      "pinned",
      "/repo/apps/oxfmt/test/.real-svelte-packages-report.json",
      "/repo/apps/oxfmt/test/.real-svelte-packages-state.json",
    );

    expect(annotations.map((annotation) => annotation.title)).toEqual(
      expect.arrayContaining([
        "Oxfmt Svelte helper manifest mismatch",
        "Oxfmt Svelte managed lane failed",
        "Oxfmt Svelte step failed: smoke",
        "Oxfmt Svelte package issue: prettier-plugin-svelte",
        "Oxfmt Svelte link issue: runtime fixture node_modules",
        "Oxfmt Svelte resolution drift: prettier-plugin-svelte",
        "Oxfmt Svelte resolution error: svelte",
        "Oxfmt Svelte npm ls reported problems",
      ]),
    );
  });

  it("emits GitHub Actions annotations with escaped properties", () => {
    process.env.GITHUB_ACTIONS = "true";

    const stdoutWrite = vi.spyOn(process.stdout, "write").mockImplementation(() => true);

    emitAnnotation({
      level: "error",
      title: "Oxfmt Svelte lane",
      message: "line 1\nline 2",
      file: "apps/oxfmt/test/.real-svelte-packages-report.json",
    });

    expect(stdoutWrite).toHaveBeenCalledTimes(1);
    expect(stdoutWrite.mock.calls[0]?.[0]).toBe(
      "::error title=Oxfmt Svelte lane,file=apps/oxfmt/test/.real-svelte-packages-report.json::line 1%0Aline 2\n",
    );
  });
});
