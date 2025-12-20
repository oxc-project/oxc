import { join } from "node:path";
import { tmpdir } from "node:os";
import fs from "node:fs/promises";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import { runCli } from "./utils";

describe("--lsp", () => {
  it("should start LSP server and respond to initialize request", async () => {
    const tempDir = await fs.mkdtemp(join(tmpdir(), "oxfmt-lsp-test"));

    const content = JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        processId: process.pid,
        capabilities: {},
        rootUri: pathToFileURL(tempDir).href,
      },
    });
    const message = `Content-Length: ${Buffer.byteLength(content)}\r\n\r\n${content}`;

    try {
      const proc = runCli(tempDir, ["--lsp"]);

      proc.stdin.write(message);
      proc.stdin.end();

      const result = await proc;
      expect(result.exitCode).toBe(0);
      // LSP server should output a response with Content-Length header
      expect(result.stdout).toContain("Content-Length:");
      // Response should contain initialize result with capabilities
      expect(result.stdout).toContain('"capabilities"');
    } finally {
      await fs.rm(tempDir, { recursive: true, force: true });
    }
  });
});
