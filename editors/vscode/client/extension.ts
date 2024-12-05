import { promises as fsPromises } from 'node:fs';

import {
  commands,
  ExtensionContext,
  Position as VsCodePosition,
  Range as VsCodeRange,
  StatusBarAlignment,
  StatusBarItem,
  ThemeColor,
  Uri,
  window,
  workspace,
  WorkspaceEdit,
} from 'vscode';

import {
  CodeAction,
  CodeActionRequest,
  CodeActionTriggerKind,
  MessageType,
  Position,
  Range,
  ShowMessageNotification,
  VersionedTextDocumentIdentifier,
} from 'vscode-languageclient';

import { Executable, LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

import { join } from 'node:path';
import { ConfigService } from './ConfigService';

const languageClientName = 'oxc';
const outputChannelName = 'Oxc';
const commandPrefix = 'oxc';

const enum OxcCommands {
  RestartServer = `${commandPrefix}.restartServer`,
  ApplyAllFixesFile = `${commandPrefix}.applyAllFixesFile`,
  ShowOutputChannel = `${commandPrefix}.showOutputChannel`,
  ToggleEnable = `${commandPrefix}.toggleEnable`,
}

let client: LanguageClient;

let myStatusBarItem: StatusBarItem;

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();
  const restartCommand = commands.registerCommand(
    OxcCommands.RestartServer,
    async () => {
      if (!client) {
        window.showErrorMessage('oxc client not found');
        return;
      }

      try {
        if (client.isRunning()) {
          await client.restart();

          window.showInformationMessage('oxc server restarted.');
        } else {
          await client.start();
        }
      } catch (err) {
        client.error('Restarting client failed', err, 'force');
      }
    },
  );

  const showOutputCommand = commands.registerCommand(
    OxcCommands.ShowOutputChannel,
    () => {
      client?.outputChannel?.show();
    },
  );

  const toggleEnable = commands.registerCommand(
    OxcCommands.ToggleEnable,
    () => {
      configService.config.updateEnable(!configService.config.enable);
    },
  );

  const applyAllFixesFile = commands.registerCommand(
    OxcCommands.ApplyAllFixesFile,
    async () => {
      if (!client) {
        window.showErrorMessage('oxc client not found');
        return;
      }
      const textEditor = window.activeTextEditor;
      if (!textEditor) {
        window.showErrorMessage('active text editor not found');
        return;
      }
      const textDocument: VersionedTextDocumentIdentifier = {
        uri: textEditor.document.uri.toString(),
        version: textEditor.document.version,
      };

      const lastLine = textEditor.document.lineAt(textEditor.document.lineCount - 1);
      const commandsOrCodeActions = await client.sendRequest(CodeActionRequest.type, {
        textDocument: {
          uri: textDocument.uri,
        },
        range: Range.create(Position.create(0, 0), lastLine.range.end),
        context: {
          diagnostics: [],
          only: [],
          triggerKind: CodeActionTriggerKind.Invoked,
        },
      }) || [];

      await Promise.all(
        commandsOrCodeActions
          // Filter to CodeActions that are preferred and contain an edit.
          .filter((commandOrCodeAction): commandOrCodeAction is CodeAction =>
            'edit' in commandOrCodeAction && !!commandOrCodeAction.edit && !!commandOrCodeAction.edit.changes &&
            !!commandOrCodeAction.isPreferred
          )
          .map(async (codeAction) => {
            const workspaceEdit = new WorkspaceEdit();

            Object.entries(codeAction.edit!.changes!).forEach(([uriString, edits]) => {
              edits.forEach(edit => {
                workspaceEdit.replace(
                  Uri.parse(uriString),
                  lspRangeToVsCodeRange(edit.range),
                  edit.newText,
                );
              });
            });
            return workspace.applyEdit(workspaceEdit);
          }),
      );
    },
  );

  context.subscriptions.push(
    applyAllFixesFile,
    restartCommand,
    showOutputCommand,
    toggleEnable,
    configService,
  );

  const outputChannel = window.createOutputChannel(outputChannelName, { log: true });

  async function findBinary(): Promise<string> {
    let bin = configService.config.binPath;
    if (bin) {
      try {
        await fsPromises.access(bin);
        return bin;
      } catch {}
    }

    const workspaceFolders = workspace.workspaceFolders;
    const isWindows = process.platform === 'win32';

    if (workspaceFolders?.length && !isWindows) {
      try {
        return await Promise.any(
          workspaceFolders.map(async (folder) => {
            const binPath = join(
              folder.uri.fsPath,
              'node_modules',
              '.bin',
              'oxc_language_server',
            );

            await fsPromises.access(binPath);
            return binPath;
          }),
        );
      } catch {}
    }

    const ext = isWindows ? '.exe' : '';
    // NOTE: The `./target/release` path is aligned with the path defined in .github/workflows/release_vscode.yml
    return (
      process.env.SERVER_PATH_DEV ??
        join(context.extensionPath, `./target/release/oxc_language_server${ext}`)
    );
  }

  const command = await findBinary();
  const run: Executable = {
    command: command!,
    options: {
      env: {
        ...process.env,
        RUST_LOG: process.env.RUST_LOG || 'info',
      },
    },
  };
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };
  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [
      'typescript',
      'javascript',
      'typescriptreact',
      'javascriptreact',
      'vue',
      'svelte',
    ].map((lang) => ({
      language: lang,
      scheme: 'file',
    })),
    synchronize: {
      // Notify the server about file config changes in the workspace
      fileEvents: [
        workspace.createFileSystemWatcher('**/.oxlint{.json,rc.json}'),
        workspace.createFileSystemWatcher('**/oxlint{.json,rc.json}'),
      ],
    },
    initializationOptions: {
      settings: configService.config.toLanguageServerConfig(),
    },
    outputChannel,
    traceOutputChannel: outputChannel,
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    languageClientName,
    serverOptions,
    clientOptions,
  );
  client.onNotification(ShowMessageNotification.type, (params) => {
    switch (params.type) {
      case MessageType.Debug:
        outputChannel.debug(params.message);
        break;
      case MessageType.Log:
        outputChannel.info(params.message);
        break;
      case MessageType.Info:
        window.showInformationMessage(params.message);
        break;
      case MessageType.Warning:
        window.showWarningMessage(params.message);
        break;
      case MessageType.Error:
        window.showErrorMessage(params.message);
        break;
      default:
        outputChannel.info(params.message);
    }
  });

  workspace.onDidDeleteFiles((event) => {
    event.files.forEach((fileUri) => {
      client.diagnostics?.delete(fileUri);
    });
  });

  configService.onConfigChange = function onConfigChange() {
    let settings = this.config.toLanguageServerConfig();
    updateStatsBar(settings.enable);
    client.sendNotification('workspace/didChangeConfiguration', { settings });
  };

  function updateStatsBar(enable: boolean) {
    if (!myStatusBarItem) {
      myStatusBarItem = window.createStatusBarItem(
        StatusBarAlignment.Right,
        100,
      );
      myStatusBarItem.command = OxcCommands.ToggleEnable;
      context.subscriptions.push(myStatusBarItem);
      myStatusBarItem.show();
    }
    let bgColor = new ThemeColor(
      enable
        ? 'statusBarItem.activeBackground'
        : 'statusBarItem.errorBackground',
    );
    myStatusBarItem.text = `oxc: ${enable ? '$(check-all)' : '$(circle-slash)'}`;

    myStatusBarItem.backgroundColor = bgColor;
  }
  updateStatsBar(configService.config.enable);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

function lspRangeToVsCodeRange(range: Range): VsCodeRange {
  return new VsCodeRange(
    new VsCodePosition(range.start.line, range.start.character),
    new VsCodePosition(range.end.line, range.end.character),
  );
}
