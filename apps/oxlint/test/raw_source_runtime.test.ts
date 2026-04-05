import { describe, expect, it } from "vitest";
import { readFile } from "node:fs/promises";
import { join } from "node:path";

const oxlintDir = join(import.meta.dirname, "..");

async function readOxlintFile(...segments: string[]): Promise<string> {
  return await readFile(join(oxlintDir, ...segments), "utf-8");
}

describe("raw source runtime support", () => {
  it("exposes #oxlint self-imports to source files through the code condition", async () => {
    const packageJson = JSON.parse(await readOxlintFile("package.json")) as {
      imports?: Record<string, unknown>;
    };

    expect(packageJson.imports).toMatchObject({
      "#oxlint": {
        code: "./src-js/index.ts",
        default: "./dist/index.js",
      },
      "#oxlint/plugins": {
        code: "./src-js/plugins.ts",
        default: "./dist-pkg-plugins/index.js",
      },
      "#oxlint/plugins-dev": {
        code: "./src-js/plugins-dev.ts",
        default: "./dist/plugins-dev.js",
      },
    });
  });

  it("keeps raw-transfer deserialization imports on source .ts modules", async () => {
    const deserialize = await readOxlintFile("src-js", "generated", "deserialize.js");

    expect(deserialize).toContain('from "../plugins/tokens.ts"');
    expect(deserialize).toContain('from "../plugins/comments.ts"');
    expect(deserialize).not.toContain('from "../plugins/tokens.js"');
    expect(deserialize).not.toContain('from "../plugins/comments.js"');
  });

  it("bootstraps runtime flags for raw-source CLI and RuleTester entrypoints", async () => {
    const [cliSource, pluginsDevSource] = await Promise.all([
      readOxlintFile("src-js", "cli.ts"),
      readOxlintFile("src-js", "plugins-dev.ts"),
    ]);

    expect(cliSource).toContain('import "./runtime_flags.ts";');
    expect(pluginsDevSource).toContain('import "./runtime_flags.ts";');
  });

  it("avoids raw-source JSON module version imports in plugin context", async () => {
    const [contextSource, packageVersionSource] = await Promise.all([
      readOxlintFile("src-js", "plugins", "context.ts"),
      readOxlintFile("src-js", "utils", "package_version.ts"),
    ]);

    expect(contextSource).toContain('from "../utils/package_version.ts"');
    expect(contextSource).not.toContain('from "../../package.json"');
    expect(packageVersionSource).toContain('resolveNearestPackageVersion(import.meta.url)');
  });

  it("marks raw-source conformance runs as CONFORMANCE builds", async () => {
    const packageJson = JSON.parse(await readOxlintFile("package.json")) as {
      scripts?: Record<string, string>;
    };

    expect(packageJson.scripts?.conformance).toContain("CONFORMANCE=true");
    expect(packageJson.scripts?.conformance).toContain(
      'tsx --conditions=code ./conformance/src/index.ts',
    );
  });

  it("keeps raw-source .js shims for generated plugin imports", async () => {
    const tokensShim = await readOxlintFile("src-js", "plugins", "tokens.js");
    const commentsShim = await readOxlintFile("src-js", "plugins", "comments.js");

    expect(tokensShim.trim()).toBe('export * from "./tokens.ts";');
    expect(commentsShim.trim()).toBe('export * from "./comments.ts";');
  });
});
