import { join, dirname } from "node:path";
import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import { TextDocument } from "vscode-languageserver-textdocument";
import {
  createMessageConnection,
  StreamMessageReader,
  StreamMessageWriter,
  InitializeRequest,
  InitializedNotification,
  DidOpenTextDocumentNotification,
  DocumentFormattingRequest,
  ShutdownRequest,
  ExitNotification,
} from "vscode-languageserver-protocol/node";
import type { TextEdit } from "vscode-languageserver-protocol/node";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures", "lsp");

function createLspConnection() {
  const cliPath = join(import.meta.dirname, "..", "dist", "cli.js");
  const proc = spawn("node", [cliPath, "--lsp"]);

  const connection = createMessageConnection(
    new StreamMessageReader(proc.stdout),
    new StreamMessageWriter(proc.stdin),
  );
  connection.listen();

  return {
    // NOTE: Config and ignore files are searched from `rootUri` upward
    // Or, provide a custom config path via `initializationOptions`
    async initialize(rootUri: string, initializationOptions?: unknown) {
      const result = await connection.sendRequest(InitializeRequest.type, {
        processId: process.pid,
        capabilities: {},
        rootUri,
        initializationOptions,
      });
      await connection.sendNotification(InitializedNotification.type, {});
      return result;
    },

    async didOpen(uri: string, languageId: string, text: string) {
      await connection.sendNotification(DidOpenTextDocumentNotification.type, {
        textDocument: { uri, languageId, version: 1, text },
      });
    },

    async format(uri: string) {
      return connection.sendRequest(DocumentFormattingRequest.type, {
        textDocument: { uri },
        // NOTE: These options are required by LSP spec, but our config will take precedence
        // https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#formattingOptions
        options: { tabSize: 2, insertSpaces: true },
      });
    },

    async [Symbol.asyncDispose]() {
      await connection.sendRequest(ShutdownRequest.type);
      await connection.sendNotification(ExitNotification.type);
      connection.dispose();
      proc.kill();
    },
  };
}

async function formatFixture(
  fixturePath: string,
  languageId: string,
  initializationOptions?: unknown,
): Promise<string> {
  const filePath = join(FIXTURES_DIR, fixturePath);
  const dirPath = dirname(filePath);
  const fileUri = pathToFileURL(filePath).href;
  const content = await fs.readFile(filePath, "utf-8");

  await using client = createLspConnection();

  await client.initialize(pathToFileURL(dirPath).href, initializationOptions);
  await client.didOpen(fileUri, languageId, content);

  const edits = await client.format(fileUri);

  return `
--- FILE -----------
${fixturePath}
--- BEFORE ---------
${content}
--- AFTER ----------
${applyEdits(content, edits, languageId)}
--------------------
`.trim();

  function applyEdits(
    content: string,
    edits: TextEdit[] | null,
    languageId: string,
  ): string | null {
    if (edits === null || edits.length === 0) return null;
    const doc = TextDocument.create("file:///test", languageId, 1, content);
    return TextDocument.applyEdits(doc, edits);
  }
}

// ---

describe("--lsp", () => {
  it("should start LSP server and respond to initialize request", async () => {
    const dirPath = join(FIXTURES_DIR, "format");
    await using client = createLspConnection();
    const initResult = await client.initialize(pathToFileURL(dirPath).href);

    expect(initResult.capabilities.documentFormattingProvider).toBe(true);
  });

  describe("formatting", () => {
    it.each([
      ["format/test.tsx", "typescriptreact"],
      ["format/test.json", "json"],
      ["format/test.vue", "vue"],
      ["format/test.toml", "toml"],
      ["format/formatted.ts", "typescript"],
      ["format/test.txt", "plaintext"],
    ])("should handle %s", async (path, languageId) => {
      expect(await formatFixture(path, languageId)).toMatchSnapshot();
    });
  });

  describe("config options", () => {
    it.each([
      ["config-semi/test.ts", "typescript"],
      ["config-no-sort-package-json/package.json", "json"],
      ["config-vue-indent/test.vue", "vue"],
      ["config-sort-imports/test.js", "javascript"],
      ["config-sort-tailwindcss/test.tsx", "typescriptreact"],
      ["config-sort-tailwindcss/test.vue", "vue"],
      ["config-sort-both/test.jsx", "javascriptreact"],
      ["editorconfig/test.ts", "typescript"],
    ])("should apply config from %s", async (path, languageId) => {
      expect(await formatFixture(path, languageId)).toMatchSnapshot();
    });
  });

  describe("ignore patterns", () => {
    it.each([
      ["ignore-prettierignore/ignored.ts", "typescript"],
      ["ignore-config/file.generated.ts", "typescript"],
    ])("should handle %s", async (path, languageId) => {
      expect(await formatFixture(path, languageId)).toMatchSnapshot();
    });

    // .gitignore is created dynamically to avoid git ignoring the test fixture
    it("should respect .gitignore", async () => {
      const testDir = join(FIXTURES_DIR, "ignore-gitignore");
      const gitignorePath = join(testDir, ".gitignore");
      const ignoredPath = join(testDir, "ignored.ts");
      const notIgnoredPath = join(testDir, "not-ignored.ts");

      try {
        await fs.mkdir(testDir, { recursive: true });
        await fs.writeFile(gitignorePath, "ignored.ts\n");
        await fs.writeFile(ignoredPath, "const   x   =   1\n");
        await fs.writeFile(notIgnoredPath, "const   x   =   1\n");

        const ignoredResult = await formatFixture("ignore-gitignore/ignored.ts", "typescript");
        const notIgnoredResult = await formatFixture(
          "ignore-gitignore/not-ignored.ts",
          "typescript",
        );

        expect(ignoredResult).toMatchSnapshot();
        expect(notIgnoredResult).toMatchSnapshot();
      } finally {
        await fs.rm(testDir, { recursive: true, force: true });
      }
    });
  });

  describe("initializationOptions", () => {
    it("should use custom config path from fmt.configPath", async () => {
      expect(
        await formatFixture("custom_config_path/semicolons-as-needed.ts", "typescript", {
          settings: {
            "fmt.configPath": "./format.json",
          },
        }),
      ).toMatchSnapshot();
    });
  });
});
