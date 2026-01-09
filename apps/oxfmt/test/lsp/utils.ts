import { join, dirname } from "node:path";
import fs from "node:fs/promises";
import { spawn } from "node:child_process";
import { pathToFileURL } from "node:url";
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
import { TextDocument } from "vscode-languageserver-textdocument";
import type { TextEdit } from "vscode-languageserver-protocol/node";

const CLI_PATH = join(import.meta.dirname, "..", "..", "dist", "cli.js");

export function createLspConnection() {
  const proc = spawn("node", [CLI_PATH, "--lsp"]);

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

// ---

export async function formatFixture(
  fixturesDir: string,
  fixturePath: string,
  languageId: string,
  initializationOptions?: unknown,
): Promise<string> {
  const filePath = join(fixturesDir, fixturePath);
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
}

// ---

function applyEdits(content: string, edits: TextEdit[] | null, languageId: string): string | null {
  if (edits === null || edits.length === 0) return null;
  const doc = TextDocument.create("file:///test", languageId, 1, content);
  return TextDocument.applyEdits(doc, edits);
}
