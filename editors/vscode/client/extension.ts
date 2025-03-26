import { promises as fsPromises } from 'node:fs';

import {
  ExtensionContext,
  RelativePattern,
  StatusBarAlignment,
  StatusBarItem,
  ThemeColor,
  window,
  workspace,
} from 'vscode';

import { MessageType, ShowMessageNotification } from 'vscode-languageclient';

import { Executable, LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

import { join } from 'node:path';
import { applyAllFixesFile, OxcCommands, restartCommand, showOutputCommand, toggleEnable } from './commands';
import { oxlintConfigFileName } from './Config';
import { ConfigService } from './ConfigService';
import { findOxlintrcConfigFiles, sendDidChangeWatchedFilesNotificationWith } from './utilities';

const languageClientName = 'oxc';
const outputChannelName = 'Oxc';

let client: LanguageClient;

let myStatusBarItem: StatusBarItem;

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();
  const clientGetter = () => client;

  context.subscriptions.push(
    applyAllFixesFile(clientGetter),
    restartCommand(clientGetter),
    showOutputCommand(clientGetter),
    toggleEnable(clientGetter, configService.config),
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
      fileEvents: createFileEventWatchers(configService.config.configPath),
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

  configService.onConfigChange = function onConfigChange(event) {
    let settings = this.config.toLanguageServerConfig();
    updateStatsBar(this.config.enable);
    client.sendNotification('workspace/didChangeConfiguration', { settings });

    if (event.affectsConfiguration('oxc.configPath')) {
      client.clientOptions.synchronize = client.clientOptions.synchronize ?? {};
      client.clientOptions.synchronize.fileEvents = createFileEventWatchers(configService.config.configPath);
      client.restart().then(async () => {
        const configFiles = await findOxlintrcConfigFiles();
        await sendDidChangeWatchedFilesNotificationWith(client, configFiles);
      });
    }
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

  if (configService.config.enable) {
    await startClient();
  }
}

// Starts the client, it does not check if it is already started
async function startClient() {
  await client.start();
  // ToDo: refactor it on the server side.
  // Do not touch watchers on client side, just simplify the start of the server.
  const configFiles = await findOxlintrcConfigFiles();
  await sendDidChangeWatchedFilesNotificationWith(client, configFiles);
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

function createFileEventWatchers(configRelativePath: string) {
  const workspaceConfigPatterns = (workspace.workspaceFolders || []).map((workspaceFolder) =>
    new RelativePattern(workspaceFolder, configRelativePath)
  );

  return [
    workspace.createFileSystemWatcher(`**/${oxlintConfigFileName}`),
    ...workspaceConfigPatterns.map((pattern) => {
      return workspace.createFileSystemWatcher(pattern);
    }),
  ];
}
