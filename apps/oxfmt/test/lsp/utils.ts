import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import {
  createMessageConnection,
  DidChangeConfigurationNotification,
  DidChangeTextDocumentNotification,
  DidOpenTextDocumentNotification,
  DocumentFormattingRequest,
  ExitNotification,
  InitializedNotification,
  InitializeRequest,
  RegistrationRequest,
  ShutdownRequest,
  StreamMessageReader,
  StreamMessageWriter,
  WorkspaceFolder,
} from "vscode-languageserver-protocol/node";
import { TextDocument } from "vscode-languageserver-textdocument";
import type {
  ClientCapabilities,
  Registration,
  TextEdit,
} from "vscode-languageserver-protocol/node";

const CLI_PATH = join(import.meta.dirname, "..", "..", "dist", "cli.js");

export function createLspConnection() {
  const proc = spawn("node", [CLI_PATH, "--lsp"], {
    // env: { ...process.env, OXC_LOG: "info" }, for debugging
  });

  const connection = createMessageConnection(
    new StreamMessageReader(proc.stdout),
    new StreamMessageWriter(proc.stdin),
  );
  connection.listen();

  return {
    // NOTE: Config and ignore files are searched from `workspaceFolders[].uri` upward
    // Or, provide a custom config path via `initializationOptions`
    async initialize(
      workspaceFolders: WorkspaceFolder[],
      capabilities: ClientCapabilities = {},
      initializationOptions?: unknown,
    ) {
      const result = await connection.sendRequest(InitializeRequest.type, {
        processId: process.pid,
        capabilities,
        workspaceFolders,
        rootUri: null,
        initializationOptions,
      });
      await connection.sendNotification(InitializedNotification.type, {});
      return result;
    },

    async didChangeConfiguration(settings: unknown) {
      await connection.sendNotification(DidChangeConfigurationNotification.type, { settings });
    },

    async didOpen(uri: string, languageId: string, text: string) {
      await connection.sendNotification(DidOpenTextDocumentNotification.type, {
        textDocument: { uri, languageId, version: 1, text },
      });
    },

    async didChange(uri: string, text: string) {
      await connection.sendNotification(DidChangeTextDocumentNotification.type, {
        textDocument: { uri, version: 2 },
        contentChanges: [{ text }],
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

    async getDynamicRegistration(): Promise<Registration[]> {
      return await new Promise((resolve) => {
        const disposer = connection.onRequest(RegistrationRequest.type, (params) => {
          resolve(params.registrations);
          disposer.dispose();
        });
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
  initializationOptions?: OxfmtLSPConfig,
): Promise<string> {
  const filePath = join(fixturesDir, fixturePath);
  const fileUri = pathToFileURL(filePath).href;

  return await formatFixtureContent(
    fixturesDir,
    fixturePath,
    fileUri,
    languageId,
    initializationOptions,
  );
}

export async function formatFixtureContent(
  fixturesDir: string,
  fixturePath: string,
  fileUri: string,
  languageId: string,
  initializationOptions?: OxfmtLSPConfig,
): Promise<string> {
  const filePath = join(fixturesDir, fixturePath);
  const dirPath = dirname(filePath);
  const content = await fs.readFile(filePath, "utf-8");

  await using client = createLspConnection();

  await client.initialize([{ uri: pathToFileURL(dirPath).href, name: "test" }], {}, [
    {
      workspaceUri: pathToFileURL(dirPath).href,
      options: initializationOptions,
    },
  ]);
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

export async function formatFixtureAfterConfigChange(
  fixturesDir: string,
  fixturePath: string,
  languageId: string,
  initializationOptions: OxfmtLSPConfig,
  configurationChangeOptions: OxfmtLSPConfig,
): Promise<string> {
  const filePath = join(fixturesDir, fixturePath);
  const dirPath = dirname(filePath);
  const fileUri = pathToFileURL(filePath).href;
  const content = await fs.readFile(filePath, "utf-8");

  await using client = createLspConnection();

  // Initial format with first config
  await client.initialize([{ uri: pathToFileURL(dirPath).href, name: "test" }], {}, [
    {
      workspaceUri: pathToFileURL(dirPath).href,
      options: initializationOptions,
    },
  ]);
  await client.didOpen(fileUri, languageId, content);
  const edits1 = await client.format(fileUri);
  const formatted1 = applyEdits(content, edits1, languageId);
  await client.didChange(fileUri, formatted1);

  // Re-format with second config
  await client.didChangeConfiguration([
    { workspaceUri: pathToFileURL(dirPath).href, options: configurationChangeOptions },
  ]);
  const edits2 = await client.format(fileUri);
  const formatted2 = applyEdits(formatted1, edits2, languageId);

  return `
--- FILE -----------
${fixturePath}
--- BEFORE ---------
${content}
--- AFTER FIRST FORMAT ----------
${formatted1}
--- AFTER SECOND FORMAT ----------
${formatted2}
--------------------
`.trim();
}

// ---

// aligned with https://github.com/oxc-project/oxc/blob/7e6c15baaebf206ab540191da0e4e103e4fabf06/apps/oxfmt/src/lsp/options.rs
type OxfmtLSPConfig = {
  "fmt.configPath"?: string | null;
};

function applyEdits(content: string, edits: TextEdit[] | null, languageId: string): string {
  if (edits === null || edits.length === 0) return content;
  const doc = TextDocument.create("file:///test", languageId, 1, content);
  return TextDocument.applyEdits(doc, edits);
}
