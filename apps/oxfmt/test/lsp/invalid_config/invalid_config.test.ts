import { mkdir, mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it, onTestFinished } from "vitest";
import { TextDocument } from "vscode-languageserver-textdocument";
import { createLspConnection } from "../utils";

const SOURCE = "const value = 1;";
const INVALID_SORT_CONFIG = JSON.stringify({ sortImports: { partitionByNewline: true } });

async function createWorkspace(config: string) {
  const root = await realpath(await mkdtemp(join(tmpdir(), "oxfmt-invalid-config-")));
  onTestFinished(() => rm(root, { recursive: true, force: true }));
  await writeFile(join(root, ".oxfmtrc.json"), config);
  return { root, uri: pathToFileURL(root).href };
}

describe("LSP invalid config", () => {
  it.each([
    ["empty JSON", "", "EOF while parsing a value"],
    ["malformed JSON", "{", "EOF while parsing an object"],
    ["invalid option type", '{"semi":"invalid"}', "expected a boolean"],
    ["invalid import sorting", INVALID_SORT_CONFIG, "partitionByNewline"],
  ])("reports %s instead of formatting with defaults", async (_name, config, message) => {
    const workspace = await createWorkspace(config);
    const uri = pathToFileURL(join(workspace.root, "example.ts")).href;
    await using client = createLspConnection();
    await client.initialize([{ uri: workspace.uri, name: "test" }]);
    await client.didOpen(uri, "typescript", SOURCE);

    await expect(client.format(uri)).rejects.toThrow(message);
  });

  it("reports invalid explicit config instead of using the workspace config", async () => {
    const workspace = await createWorkspace('{"semi":false}');
    await writeFile(join(workspace.root, "format.json"), INVALID_SORT_CONFIG);
    const uri = pathToFileURL(join(workspace.root, "example.ts")).href;
    await using client = createLspConnection();
    await client.initialize([{ uri: workspace.uri, name: "test" }], {}, [
      { workspaceUri: workspace.uri, options: { "fmt.configPath": "./format.json" } },
    ]);
    await client.didOpen(uri, "typescript", SOURCE);

    await expect(client.format(uri)).rejects.toThrow("partitionByNewline");
  });

  it("reports invalid nested config without changing other scopes", async () => {
    const workspace = await createWorkspace('{"semi":false}');
    const nested = join(workspace.root, "nested");
    await mkdir(nested);
    await writeFile(join(nested, ".oxfmtrc.json"), INVALID_SORT_CONFIG);
    const nestedUri = pathToFileURL(join(nested, "example.ts")).href;
    const rootUri = pathToFileURL(join(workspace.root, "example.ts")).href;
    await using client = createLspConnection();
    await client.initialize([{ uri: workspace.uri, name: "test" }]);
    await client.didOpen(nestedUri, "typescript", SOURCE);
    await client.didOpen(rootUri, "typescript", SOURCE);

    await expect(client.format(nestedUri)).rejects.toThrow("partitionByNewline");
    const edits = await client.format(rootUri);
    expect(
      TextDocument.applyEdits(TextDocument.create(rootUri, "typescript", 1, SOURCE), edits ?? []),
    ).toBe("const value = 1\n");
  });

  it("reports invalid config for an in-memory document", async () => {
    const workspace = await createWorkspace(INVALID_SORT_CONFIG);
    const uri = "untitled://Untitled-1";
    await using client = createLspConnection();
    await client.initialize([{ uri: workspace.uri, name: "test" }]);
    await client.didOpen(uri, "typescript", SOURCE);

    await expect(client.format(uri)).rejects.toThrow("partitionByNewline");
  });
});
