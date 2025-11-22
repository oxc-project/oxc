import { promises as fsPromises } from 'node:fs';

import { commands, ConfigurationChangeEvent, ExtensionContext, LogOutputChannel, Uri, window, workspace } from 'vscode';

import { ConfigurationParams, ExecuteCommandRequest, ShowMessageNotification } from 'vscode-languageclient';

import { Executable, LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

import { join } from 'node:path';
import { OxcCommands } from './commands';
import { ConfigService } from './ConfigService';
import { onClientNotification, runExecutable } from './lsp_helper';
import StatusBarItemHandler from './StatusBarItemHandler';
import { VSCodeConfig } from './VSCodeConfig';

const languageClientName = 'oxc';

const enum LspCommands {
  FixAll = 'oxc.fixAll',
}

let client: LanguageClient | undefined;

// Global flag to check if the user allows us to start the server.
// When `oxc.requireConfig` is `true`, make sure one `.oxlintrc.json` file is present.
let allowedToStartServer: boolean;

export async function activate(
  context: ExtensionContext,
  outputChannel: LogOutputChannel,
  configService: ConfigService,
  statusBarItemHandler: StatusBarItemHandler,
) {
  allowedToStartServer = configService.vsCodeConfig.requireConfig
    ? (await workspace.findFiles(`**/.oxlintrc.json`, '**/node_modules/**', 1)).length > 0
    : true;

  const applyAllFixesFile = commands.registerCommand(OxcCommands.ApplyAllFixesFile, async () => {
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
      arguments: [
        {
          uri: textEditor.document.uri.toString(),
        },
      ],
    };

    await client.sendRequest(ExecuteCommandRequest.type, params);
  });

  context.subscriptions.push(applyAllFixesFile);

  async function findBinary(): Promise<string> {
    const bin = configService.getUserServerBinPath();
    if (workspace.isTrusted && bin) {
      try {
        await fsPromises.access(bin);
        return bin;
      } catch (e) {
        outputChannel.error(`Invalid bin path: ${bin}`, e);
      }
    }
    const ext = process.platform === 'win32' ? '.exe' : '';
    // NOTE: The `./target/release` path is aligned with the path defined in .github/workflows/release_vscode.yml
    return process.env.SERVER_PATH_DEV ?? join(context.extensionPath, `./target/release/oxc_language_server${ext}`);
  }

  const path = await findBinary();

  const run: Executable = runExecutable(path, configService.vsCodeConfig.nodePath);
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };

  outputChannel.info(`Using server binary at: ${path}`);

  // see https://github.com/oxc-project/oxc/blob/9b475ad05b750f99762d63094174be6f6fc3c0eb/crates/oxc_linter/src/loader/partial_loader/mod.rs#L17-L20
  const supportedExtensions = ['astro', 'cjs', 'cts', 'js', 'jsx', 'mjs', 'mts', 'svelte', 'ts', 'tsx', 'vue'];

  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [
      {
        pattern: `**/*.{${supportedExtensions.join(',')}}`,
        scheme: 'file',
      },
    ],
    initializationOptions: configService.languageServerConfig,
    outputChannel,
    traceOutputChannel: outputChannel,
    middleware: {
      handleDiagnostics: (uri, diagnostics, next) => {
        for (const diag of diagnostics) {
          // https://github.com/oxc-project/oxc/issues/12404
          if (typeof diag.code === 'object' && diag.code?.value === 'eslint-plugin-unicorn(filename-case)') {
            diag.message += '\nYou may need to close the file and restart VSCode after renaming a file by only casing.';
          }
        }
        next(uri, diagnostics);
      },
      workspace: {
        configuration: (params: ConfigurationParams) => {
          return params.items.map((item) => {
            if (item.section !== 'oxc_language_server') {
              return null;
            }
            if (item.scopeUri === undefined) {
              return null;
            }

            return configService.getWorkspaceConfig(Uri.parse(item.scopeUri))?.toLanguageServerConfig() ?? null;
          });
        },
      },
    },
  };

  // Create the language client and start the client.
  client = new LanguageClient(languageClientName, serverOptions, clientOptions);

  const onNotificationDispose = client.onNotification(ShowMessageNotification.type, (params) => {
    onClientNotification(params, outputChannel);
  });

  context.subscriptions.push(onNotificationDispose);

  const onDeleteFilesDispose = workspace.onDidDeleteFiles((event) => {
    for (const fileUri of event.files) {
      client?.diagnostics?.delete(fileUri);
    }
  });

  context.subscriptions.push(onDeleteFilesDispose);

  updateStatusBar(statusBarItemHandler, configService.vsCodeConfig.enable);
  if (allowedToStartServer) {
    if (configService.vsCodeConfig.enable) {
      await client.start();
    }
  } else {
    generateActivatorByConfig(configService.vsCodeConfig, context, statusBarItemHandler);
  }
}

export async function deactivate(): Promise<void> {
  if (!client) {
    return undefined;
  }
  await client.stop();
  client = undefined;
}

/**
 * Get the status bar state based on whether oxc is enabled and allowed to start.
 */
function getStatusBarState(enable: boolean): { bgColor: string; icon: string; tooltipText: string } {
  if (!allowedToStartServer) {
    return {
      bgColor: 'statusBarItem.offlineBackground',
      icon: 'circle-slash',
      tooltipText: 'oxc is disabled (no .oxlintrc.json found)',
    };
  } else if (!enable) {
    return { bgColor: 'statusBarItem.warningBackground', icon: 'check', tooltipText: 'oxc is disabled' };
  } else {
    return { bgColor: 'statusBarItem.activeBackground', icon: 'check-all', tooltipText: 'oxc is enabled' };
  }
}

function updateStatusBar(statusBarItemHandler: StatusBarItemHandler, enable: boolean) {
  const { bgColor, icon, tooltipText } = getStatusBarState(enable);

  let text =
    `**${tooltipText}**\n\n` +
    `[$(terminal) Open Output](command:${OxcCommands.ShowOutputChannel})\n\n` +
    `[$(refresh) Restart Server](command:${OxcCommands.RestartServer})\n\n`;

  if (enable) {
    text += `[$(stop) Stop Server](command:${OxcCommands.ToggleEnable})\n\n`;
  } else {
    text += `[$(play) Start Server](command:${OxcCommands.ToggleEnable})\n\n`;
  }

  statusBarItemHandler.setColorAndIcon(bgColor, icon);
  statusBarItemHandler.updateToolTooltip('linter', text);
}

function generateActivatorByConfig(
  config: VSCodeConfig,
  context: ExtensionContext,
  statusBarItemHandler: StatusBarItemHandler,
): void {
  const watcher = workspace.createFileSystemWatcher('**/.oxlintrc.json', false, true, !config.requireConfig);
  watcher.onDidCreate(async () => {
    allowedToStartServer = true;
    updateStatusBar(statusBarItemHandler, config.enable);
    if (client && !client.isRunning() && config.enable) {
      await client.start();
    }
  });

  watcher.onDidDelete(async () => {
    // only can be called when config.requireConfig
    allowedToStartServer = (await workspace.findFiles(`**/.oxlintrc.json`, '**/node_modules/**', 1)).length > 0;
    if (!allowedToStartServer) {
      updateStatusBar(statusBarItemHandler, false);
      if (client && client.isRunning()) {
        await client.stop();
      }
    }
  });

  context.subscriptions.push(watcher);
}

export async function restartClient(): Promise<void> {
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
}

export async function toggleClient(configService: ConfigService): Promise<void> {
  if (client === undefined || !allowedToStartServer) {
    return;
  }

  if (client.isRunning()) {
    if (!configService.vsCodeConfig.enable) {
      await client.stop();
    }
  } else {
    if (configService.vsCodeConfig.enable) {
      await client.start();
    }
  }
}

export async function onConfigChange(
  event: ConfigurationChangeEvent,
  configService: ConfigService,
  statusBarItemHandler: StatusBarItemHandler,
): Promise<void> {
  updateStatusBar(statusBarItemHandler, configService.vsCodeConfig.enable);

  if (client === undefined) {
    return;
  }

  // update the initializationOptions for a possible restart
  client.clientOptions.initializationOptions = configService.languageServerConfig;

  if (configService.effectsWorkspaceConfigChange(event) && client.isRunning()) {
    await client.sendNotification('workspace/didChangeConfiguration', {
      settings: configService.languageServerConfig,
    });
  }
}
