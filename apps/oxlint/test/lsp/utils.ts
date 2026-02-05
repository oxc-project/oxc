import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import {
  CodeActionRequest,
  createMessageConnection,
  DiagnosticSeverity,
  DidChangeConfigurationNotification,
  DidChangeTextDocumentNotification,
  DidOpenTextDocumentNotification,
  DocumentDiagnosticRequest,
  ExitNotification,
  InitializedNotification,
  InitializeRequest,
  RegistrationRequest,
  ShutdownRequest,
  StreamMessageReader,
  StreamMessageWriter,
  WorkspaceFolder,
} from "vscode-languageserver-protocol/node";
import type {
  ClientCapabilities,
  CodeAction,
  Command,
  DocumentDiagnosticReport,
  Range,
  Registration,
} from "vscode-languageserver-protocol/node";
import { TextDocument } from "vscode-languageserver-textdocument";
import { codeFrameColumns } from "@babel/code-frame";

const CLI_PATH = join(import.meta.dirname, "..", "..", "dist", "cli.js");

const PULL_DIAGNOSTICS_CAPABILITY = {
  textDocument: {
    diagnostic: {},
  },
  workspace: {
    diagnostics: {
      refreshSupport: true,
    },
  },
};

export function createLspConnection() {
  const proc = spawn(process.execPath, [CLI_PATH, "--lsp"], {
    env: {
      ...process.env,
      OXC_LOG: "debug",
    },
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

    async diagnostic(uri: string): Promise<DocumentDiagnosticReport> {
      const result = await connection.sendRequest(DocumentDiagnosticRequest.type, {
        textDocument: { uri },
      });
      return result;
    },

    async codeAction(uri: string, range: Range): Promise<(Command | CodeAction)[] | null> {
      const result = await connection.sendRequest(CodeActionRequest.type, {
        textDocument: { uri },
        range,
        context: {
          diagnostics: [],
        },
      });
      return result;
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

export async function lintFixture(
  fixturesDir: string,
  fixturePath: string,
  languageId: string,
  initializationOptions?: OxlintLSPConfig,
): Promise<string> {
  return lintMultiWorkspaceFixture(
    fixturesDir,
    [{ path: fixturePath, languageId }],
    initializationOptions ? [initializationOptions] : undefined,
  );
}

export async function lintMultiWorkspaceFixture(
  fixturesDir: string,
  fixturePaths: {
    path: string;
    languageId: string;
  }[],
  initializationOptions?: OxlintLSPConfig[],
): Promise<string> {
  const workspaceUris = fixturePaths.map(
    ({ path }) => pathToFileURL(dirname(join(fixturesDir, path))).href,
  );
  await using client = createLspConnection();

  await client.initialize(
    workspaceUris.map((uri, index) => ({ uri, name: `workspace-${index}` })),
    PULL_DIAGNOSTICS_CAPABILITY,
    workspaceUris.map((workspaceUri, index) => ({
      workspaceUri,
      options: initializationOptions?.[index] ?? null,
    })),
  );

  const snapshots = [];
  for (const fixturePath of fixturePaths) {
    snapshots.push(
      // oxlint-disable-next-line no-await-in-loop -- for snapshot consistency
      await getDiagnosticSnapshot(
        fixturePath.path,
        join(fixturesDir, fixturePath.path),
        fixturePath.languageId,
        client,
      ),
    );
  }

  return snapshots.join("\n\n");
}

export async function fixFixture(
  fixturesDir: string,
  fixturePath: string,
  languageId: string,
  initializationOptions?: OxlintLSPConfig,
): Promise<string> {
  return fixMultiWorkspaceFixture(
    fixturesDir,
    [{ path: fixturePath, languageId }],
    initializationOptions ? [initializationOptions] : undefined,
  );
}

export async function fixMultiWorkspaceFixture(
  fixturesDir: string,
  fixturePaths: {
    path: string;
    languageId: string;
  }[],
  initializationOptions?: OxlintLSPConfig[],
): Promise<string> {
  const workspaceUris = fixturePaths.map(
    ({ path }) => pathToFileURL(dirname(join(fixturesDir, path))).href,
  );
  await using client = createLspConnection();

  await client.initialize(
    workspaceUris.map((uri, index) => ({ uri, name: `workspace-${index}` })),
    PULL_DIAGNOSTICS_CAPABILITY,
    workspaceUris.map((workspaceUri, index) => ({
      workspaceUri,
      options: initializationOptions?.[index] ?? null,
    })),
  );

  const snapshots = [];
  for (const fixturePath of fixturePaths) {
    snapshots.push(
      // oxlint-disable-next-line no-await-in-loop -- for snapshot consistency
      await getCodeActionSnapshot(
        fixturePath.path,
        join(fixturesDir, fixturePath.path),
        fixturePath.languageId,
        client,
      ),
    );
  }

  return snapshots.join("\n\n");
}
// ---

type OxlintLSPConfig = {
  fixKind?: string;
  configPath?: string;
};

async function getDiagnosticSnapshot(
  fixturePath: string,
  filePath: string,
  languageId: string,
  client: ReturnType<typeof createLspConnection>,
): Promise<string> {
  const fileUri = pathToFileURL(filePath).href;
  const content = await fs.readFile(filePath, "utf-8");

  await client.didOpen(fileUri, languageId, content);

  const diagnostics = await client.diagnostic(fileUri);

  return `
--- FILE -----------
${fixturePath}
--- Diagnostics ---------
${snapshotDiagnostics(content, diagnostics).join("\n--------------------")}
--------------------
`.trim();
}

async function getCodeActionSnapshot(
  fixturePath: string,
  filePath: string,
  languageId: string,
  client: ReturnType<typeof createLspConnection>,
): Promise<string> {
  const fileUri = pathToFileURL(filePath).href;
  const content = await fs.readFile(filePath, "utf-8");

  await client.didOpen(fileUri, languageId, content);

  const diagnostics = await client.diagnostic(fileUri);
  if (diagnostics.kind !== "full") {
    throw new Error("Only full reports are supported by oxlint lsp");
  }

  const codeActions = [];
  for (const diagnostic of diagnostics.items) {
    // oxlint-disable-next-line no-await-in-loop -- for snapshot consistency
    const actions = await client.codeAction(fileUri, diagnostic.range);
    if (!actions) continue;

    for (const action of actions) {
      if (!("edit" in action)) {
        throw new Error("Only code actions with edits are supported by oxlint lsp");
      }
      const result = snapshotCodeAction(fileUri, content, languageId, action);
      codeActions.push(result);
    }
  }

  return `
--- FILE -----------
${fixturePath}
--- Original Content ---------
${content}
--- Code Actions ---------
${codeActions.join("######\n")}
--------------------
`.trim();
}

function getSeverityLabel(severity: number | undefined): string {
  if (!severity) return "Unknown";

  switch (severity) {
    case DiagnosticSeverity.Error:
      return "Error";
    case DiagnosticSeverity.Warning:
      return "Warning";
    case DiagnosticSeverity.Information:
      return "Information";
    case DiagnosticSeverity.Hint:
      return "Hint";
    default:
      return "Unknown";
  }
}

function lspRangeToBabelLocation(range: Range) {
  return {
    start: {
      line: range.start.line + 1,
      column: range.start.character + 1,
    },
    end: {
      line: range.end.line + 1,
      column: range.end.character + 1,
    },
  };
}

function snapshotDiagnostics(content: string, report: DocumentDiagnosticReport): string[] {
  if (report.kind !== "full") {
    throw new Error("Only full reports are supported by oxlint lsp");
  }

  return report.items.map((diagnostic) => {
    const babelLocation = lspRangeToBabelLocation(diagnostic.range);
    const severity = getSeverityLabel(diagnostic.severity);

    return codeFrameColumns(content, babelLocation, {
      message: `${severity}: ${diagnostic.message}`,
    });
  });
}

function snapshotCodeAction(
  uri: string,
  content: string,
  languageId: string,
  action: CodeAction,
): string {
  if (!action.edit) {
    throw new Error("Only code actions with edits are supported by oxlint lsp");
  }

  let result = content;
  for (const [editedUri, textEdits] of Object.entries(action.edit.changes ?? {})) {
    if (editedUri !== uri) {
      throw new Error("Only edits for the same document are supported by oxlint lsp");
    }

    const doc = TextDocument.create(uri, languageId, 1, content);
    result = TextDocument.applyEdits(doc, textEdits);
  }

  return `
Title : ${action.title}
Preferred: ${action.isPreferred ? "Yes" : "No"}
Resulting Content:
${result}`;
}
