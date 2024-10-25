import { promises as fsPromises } from 'node:fs';

import { commands, ExtensionContext, StatusBarAlignment, StatusBarItem, ThemeColor, window, workspace } from 'vscode';

import { Executable, LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

import { join } from 'node:path';
import { ConfigService } from './config';

const languageClientId = 'oxc-vscode';
const languageClientName = 'oxc';
const outputChannelName = 'Oxc';
const traceOutputChannelName = 'Oxc (Trace)';
const commandPrefix = 'oxc';

const enum OxcCommands {
  RestartServer = `${commandPrefix}.restartServer`,
  ApplyAllFixes = `${commandPrefix}.applyAllFixes`,
  ShowOutputChannel = `${commandPrefix}.showOutputChannel`,
  ShowTraceOutputChannel = `${commandPrefix}.showTraceOutputChannel`,
  ToggleEnable = `${commandPrefix}.toggleEnable`,
}

let client: LanguageClient;

let myStatusBarItem: StatusBarItem;

export async function activate(context: ExtensionContext) {
  const config = new ConfigService();
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

  const showTraceOutputCommand = commands.registerCommand(
    OxcCommands.ShowTraceOutputChannel,
    () => {
      client?.traceOutputChannel?.show();
    },
  );

  const toggleEnable = commands.registerCommand(
    OxcCommands.ToggleEnable,
    () => {
      config.enable = !config.enable;
    },
  );

  context.subscriptions.push(
    restartCommand,
    showOutputCommand,
    showTraceOutputCommand,
    toggleEnable,
    config,
  );

  const outputChannel = window.createOutputChannel(outputChannelName);
  const traceOutputChannel = window.createOutputChannel(traceOutputChannelName);

  async function findBinary(): Promise<string> {
    let bin = config.binPath;
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
      // Notify the server about file changes to '.clientrc files contained in the workspace
      fileEvents: workspace.createFileSystemWatcher('**/.clientrc'),
    },
    initializationOptions: {
      settings: config.toLanguageServerConfig(),
    },
    outputChannel,
    traceOutputChannel,
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    languageClientId,
    languageClientName,
    serverOptions,
    clientOptions,
  );

  workspace.onDidDeleteFiles((event) => {
    event.files.forEach((fileUri) => {
      client.diagnostics?.delete(fileUri);
    });
  });

  config.onConfigChange = function onConfigChange() {
    let settings: any = JSON.parse(JSON.stringify(this));
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
  updateStatsBar(config.enable);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
