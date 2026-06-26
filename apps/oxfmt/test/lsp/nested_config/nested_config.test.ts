import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import { createLspConnection, formatFixture } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

describe("LSP nested config", () => {
  it("should apply different configs per directory", async () => {
    const rootDir = join(FIXTURES_DIR, "nested-config");
    const rootUri = pathToFileURL(rootDir).href;
    await using client = createLspConnection();
    await client.initialize([{ uri: rootUri, name: "test" }], {}, [
      { workspaceUri: rootUri, options: null },
    ]);

    // root.ts: root config (semi: true, singleQuote: true)
    expect(
      await formatFixture(FIXTURES_DIR, "nested-config/root.ts", "typescript", client),
    ).toMatchSnapshot();
    // sub/test.ts: nested config (semi: false, singleQuote: false)
    expect(
      await formatFixture(FIXTURES_DIR, "nested-config/sub/test.ts", "typescript", client),
    ).toMatchSnapshot();
  });

  it("should fall back to root config when sub dir has no config", async () => {
    const rootDir = join(FIXTURES_DIR, "nested-config");
    const rootUri = pathToFileURL(rootDir).href;
    await using client = createLspConnection();
    await client.initialize([{ uri: rootUri, name: "test" }], {}, [
      { workspaceUri: rootUri, options: null },
    ]);

    expect(
      await formatFixture(
        FIXTURES_DIR,
        "nested-config/no-config-sub/test.ts",
        "typescript",
        client,
      ),
    ).toMatchSnapshot();
  });

  it("should respect per-scope ignorePatterns from nested config", async () => {
    const rootDir = join(FIXTURES_DIR, "nested-config");
    const rootUri = pathToFileURL(rootDir).href;
    await using client = createLspConnection();
    await client.initialize([{ uri: rootUri, name: "test" }], {}, [
      { workspaceUri: rootUri, options: null },
    ]);

    expect(
      await formatFixture(
        FIXTURES_DIR,
        "nested-config/sub/ignored.generated.ts",
        "typescript",
        client,
      ),
    ).toMatchSnapshot();
  });

  it("should ignore nested config when explicit configPath is set", async () => {
    const rootDir = join(FIXTURES_DIR, "nested-config-explicit");
    const rootUri = pathToFileURL(rootDir).href;
    await using client = createLspConnection();
    await client.initialize([{ uri: rootUri, name: "test" }], {}, [
      { workspaceUri: rootUri, options: { "fmt.configPath": "./explicit.json" } },
    ]);

    // sub has singleQuote: false, but explicit config has singleQuote: true
    expect(
      await formatFixture(FIXTURES_DIR, "nested-config-explicit/sub/test.ts", "typescript", client),
    ).toMatchSnapshot();
  });
});
