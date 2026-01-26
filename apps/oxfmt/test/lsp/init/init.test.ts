import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import { createLspConnection } from "../utils";
import { WatchKind } from "vscode-languageserver-protocol/lib/node/main";

describe("LSP initialization", () => {
  it("should start LSP server and respond to initialize request", async () => {
    const dirPath = import.meta.dirname;
    await using client = createLspConnection();
    const initResult = await client.initialize([
      { uri: pathToFileURL(dirPath).href, name: "test" },
    ]);

    expect(initResult.capabilities.documentFormattingProvider).toBe(true);
    expect(initResult.serverInfo?.name).toBe("oxfmt");
  });

  it.each([
    [undefined, [".oxfmtrc.json", ".oxfmtrc.jsonc", ".editorconfig"]],
    [{ "fmt.configPath": "" }, [".oxfmtrc.json", ".oxfmtrc.jsonc", ".editorconfig"]],
    [{ "fmt.configPath": "./custom-config.json" }, ["./custom-config.json", ".editorconfig"]],
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
          id: `watcher-formatter-${dirUri}`,
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
