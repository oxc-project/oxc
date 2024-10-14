import { promises as fsPromises } from 'node:fs';

import {
  commands,
  ConfigurationTarget,
  ExtensionContext,
  StatusBarAlignment,
  StatusBarItem,
  ThemeColor,
  window,
  workspace,
} from 'vscode';

import { Executable, LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

import { join } from 'node:path';

const languageClientId = 'oxc-vscode';
const languageClientName = 'oxc';
const outputChannelName = 'oxc_language_server';
const traceOutputChannelName = 'oxc_language_server.trace';

const enum OxcCommands {
  RestartServer = 'oxc.restartServer',
  ApplyAllFixes = 'oxc.applyAllFixes',
  ShowOutputChannel = 'oxc.showOutputChannel',
  ShowTraceOutputChannel = 'oxc.showTraceOutputChannel',
  ToggleEnable = 'oxc.toggleEnable',
}

let client: LanguageClient;

let myStatusBarItem: StatusBarItem;

export async function activate(context: ExtensionContext) {
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
      let enabled = workspace
        .getConfiguration('oxc_language_server')
        .get('enable');
      let nextState = !enabled;
      workspace
        .getConfiguration('oxc_language_server')
        .update('enable', nextState, ConfigurationTarget.Global);
    },
  );

  context.subscriptions.push(
    restartCommand,
    showOutputCommand,
    showTraceOutputCommand,
    toggleEnable,
  );

  const outputChannel = window.createOutputChannel(outputChannelName);
  const traceOutputChannel = window.createOutputChannel(traceOutputChannelName);

  async function findBinary(): Promise<string> {
    const cfg = workspace.getConfiguration('oxc');

    let bin = cfg.get<string>('binPath', '');
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
  let clientConfig: any = JSON.parse(
    JSON.stringify(workspace.getConfiguration('oxc_language_server')),
  );
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
      settings: clientConfig,
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

  workspace.onDidChangeConfiguration((e) => {
    let isAffected = e.affectsConfiguration('oxc_language_server');
    if (!isAffected) {
      return;
    }
    let settings: any = JSON.parse(
      JSON.stringify(workspace.getConfiguration('oxc_language_server')),
    );
    updateStatsBar(settings.enable);
    client.sendNotification('workspace/didChangeConfiguration', { settings });
  });

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
  updateStatsBar(clientConfig.enable);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
