import { promises as fsPromises } from 'node:fs';

import {
  commands,
  ExtensionContext,
  RelativePattern,
  StatusBarAlignment,
  StatusBarItem,
  ThemeColor,
  window,
  workspace,
} from 'vscode';

import { ExecuteCommandRequest, MessageType, ShowMessageNotification } from 'vscode-languageclient';

import { Executable, LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

import { join } from 'node:path';
import { oxlintConfigFileName } from './Config';
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

const enum LspCommands {
  FixAll = 'oxc.fixAll',
}

let client: LanguageClient | undefined;

let myStatusBarItem: StatusBarItem;

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();
  const restartCommand = commands.registerCommand(
    OxcCommands.RestartServer,
    async () => {
      if (client === undefined) {
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
    async () => {
      await configService.config.updateEnable(!configService.config.enable);

      if (client === undefined) {
        return;
      }

      if (client.isRunning()) {
        if (!configService.config.enable) {
          await client.stop();
        }
      } else {
        if (configService.config.enable) {
          await client.start();
        }
      }
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

      const params = {
        command: LspCommands.FixAll,
        arguments: [{
          uri: textEditor.document.uri.toString(),
        }],
      };

      await client.sendRequest(ExecuteCommandRequest.type, params);
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
      } catch (e) {
        outputChannel.error(`Invalid bin path: ${bin}`, e);
      }
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

  const fileWatchers = createFileEventWatchers(configService.config.configPath);
  context.subscriptions.push(...fileWatchers);

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
      fileEvents: fileWatchers,
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
    for (const fileUri of event.files) {
      client?.diagnostics?.delete(fileUri);
    }
  });

  configService.onConfigChange = async function onConfigChange(event) {
    let settings = this.config.toLanguageServerConfig();
    updateStatsBar(this.config.enable);

    if (client === undefined) {
      return;
    }

    // update the initializationOptions for a possible restart
    client.clientOptions.initializationOptions = { settings };

    if (event.affectsConfiguration('oxc.configPath')) {
      client.clientOptions.synchronize = client.clientOptions.synchronize ?? {};
      client.clientOptions.synchronize.fileEvents = createFileEventWatchers(configService.config.configPath);

      if (client.isRunning()) {
        await client.restart();
      }
    } else if (client.isRunning()) {
      client.sendNotification('workspace/didChangeConfiguration', { settings });
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
        : 'statusBarItem.warningBackground',
    );
    myStatusBarItem.text = `oxc: ${enable ? '$(check-all)' : '$(check)'}`;

    myStatusBarItem.backgroundColor = bgColor;
  }
  updateStatsBar(configService.config.enable);

  if (configService.config.enable) {
    await client.start();
  }
}

export async function deactivate(): Promise<void> {
  if (!client) {
    return undefined;
  }
  await client.stop();
  client = undefined;
}

// FileSystemWatcher are not ready on the start and can take some seconds on bigger repositories
// ToDo: create test to make sure this will never break
function createFileEventWatchers(configRelativePath: string | null) {
  if (configRelativePath !== null) {
    return (workspace.workspaceFolders || []).map((workspaceFolder) =>
      workspace.createFileSystemWatcher(new RelativePattern(workspaceFolder, configRelativePath))
    );
  }

  return [
    workspace.createFileSystemWatcher(`**/${oxlintConfigFileName}`),
  ];
}
