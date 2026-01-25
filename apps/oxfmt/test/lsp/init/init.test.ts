import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import { createLspConnection } from "../utils";

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
});
