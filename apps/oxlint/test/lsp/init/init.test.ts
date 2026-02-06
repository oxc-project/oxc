import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import { createLspConnection } from "../utils";
import { WatchKind } from "vscode-languageserver-protocol/node";

describe("LSP initialization", () => {
  it("should start LSP server and respond to initialize request", async () => {
    const dirPath = import.meta.dirname;
    await using client = createLspConnection();
    const initResult = await client.initialize([
      { uri: pathToFileURL(dirPath).href, name: "test" },
    ]);

    expect(initResult.capabilities.diagnosticProvider).toBeUndefined();
    expect(initResult.serverInfo?.name).toBe("oxlint");
  });

  it("should start LSP server with diagnostics provider", async () => {
    const dirPath = import.meta.dirname;
    await using client = createLspConnection();
    const initResult = await client.initialize(
      [{ uri: pathToFileURL(dirPath).href, name: "test" }],
      {
        textDocument: {
          diagnostic: {},
        },
        workspace: {
          diagnostics: {
            refreshSupport: true,
          },
        },
      },
    );

    expect(initResult.capabilities.diagnosticProvider).toBeDefined();
    expect(initResult.serverInfo?.name).toBe("oxlint");
  });

  it.each([
    [undefined, ["**/.oxlintrc.json", "**/oxlint.config.ts"]],
    [{ configPath: "" }, ["**/.oxlintrc.json", "**/oxlint.config.ts"]],
    [{ configPath: "./custom-config.json" }, ["./custom-config.json"]],
  ])(
    "should send correct dynamic watch pattern registration for config: %s",
    async (lspConfig, expectedPatterns) => {
      const dirUri = pathToFileURL(import.meta.dirname).href;
      await using client = createLspConnection();
      await client.initialize(
        [{ uri: dirUri, name: "test" }],
        {
          workspace: {
            didChangeWatchedFiles: {
              dynamicRegistration: true,
            },
          },
        },
        [{ workspaceUri: dirUri, options: lspConfig }],
      );
      const registrations = await client.getDynamicRegistration();
      expect(registrations).toEqual([
        {
          id: `watcher-linter-${dirUri}`,
          method: "workspace/didChangeWatchedFiles",
          registerOptions: {
            watchers: expectedPatterns.map((pattern) => ({
              globPattern: {
                baseUri: dirUri,
                pattern,
              },
              kind: WatchKind.Create | WatchKind.Change | WatchKind.Delete,
            })),
          },
        },
      ]);
    },
  );
});
