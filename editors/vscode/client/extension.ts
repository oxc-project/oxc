import { promises as fsPromises } from 'node:fs';

import {
  commands,
  ExtensionContext,
  StatusBarAlignment,
  StatusBarItem,
  ThemeColor,
  Uri,
  window,
  workspace,
} from 'vscode';

import {
  ConfigurationParams,
  ExecuteCommandRequest,
  MessageType,
  ShowMessageNotification,
  State,
} from 'vscode-languageclient';

import { Executable, LanguageClient, LanguageClientOptions, ServerOptions, StreamInfo } from 'vscode-languageclient/node';
import * as net from 'node:net';

import { join } from 'node:path';
import { ConfigService } from './ConfigService';
import { VSCodeConfig } from './VSCodeConfig';

const languageClientName = 'oxc';
const outputChannelName = 'Oxc';
const commandPrefix = 'oxc';

const RESTART_DELAY_MS = 50;

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

// Global flag to check if the user allows us to start the server.
// When `oxc.requireConfig` is `true`, make sure one `.oxlintrc.json` file is present.
let allowedToStartServer: boolean;

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();
  allowedToStartServer = configService.vsCodeConfig.requireConfig
    ? (await workspace.findFiles(`**/.oxlintrc.json`, '**/node_modules/**', 1)).length > 0
    : true;

  const restartCommand = commands.registerCommand(OxcCommands.RestartServer, async () => {
    if (client === undefined) {
      window.showErrorMessage('oxc client not found');
      return;
    }

      try {

        const state = (client as LanguageClient)?.state;
        if (state === State.Starting) {
          window.showWarningMessage('oxc server is still starting; try restart again in a moment.');
          return;
        }

        if (client.isRunning()) {
          const externalSocketSpec = process.env.OXC_LS_CONNECT;
          // Use stop()+start() instead of restart() to avoid shutdown while starting error.
          if (externalSocketSpec) {
            // External socket: stop sends shutdown/exit internally; wait a tick for server loop.
            await client.stop();
            await new Promise(r => setTimeout(r, RESTART_DELAY_MS));
            await client.start();
          } else {
            // Spawned process: restart() is sufficient, but guard against transitional state.
            await client.restart();
          }
          window.showInformationMessage('oxc server restarted.');
        } else {
          // Not running (stopped) -> start it.
          await client.start();
        }
      } catch (err) {
        client.error('Restarting client failed', err, 'force');
      }
    },
  );

  const showOutputCommand = commands.registerCommand(OxcCommands.ShowOutputChannel, () => {
    client?.outputChannel?.show();
  });

  const toggleEnable = commands.registerCommand(OxcCommands.ToggleEnable, async () => {
    await configService.vsCodeConfig.updateEnable(!configService.vsCodeConfig.enable);

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
  });

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

  const outputChannel = window.createOutputChannel(outputChannelName, { log: true });

  context.subscriptions.push(
    applyAllFixesFile,
    restartCommand,
    showOutputCommand,
    toggleEnable,
    configService,
    outputChannel,
  );

  async function findBinary(): Promise<string> {
    // 1. User configured path
    const userBin = configService.getUserServerBinPath();
    if (workspace.isTrusted && userBin) {
      try {
        await fsPromises.access(userBin);
        outputChannel.info(`Using user configured oxc_language_server: ${userBin}`);
        return userBin;
      } catch (e) {
        outputChannel.error(`Configured oxc.path.server not accessible: ${userBin}`, e);
      }
    }

    const ext = process.platform === 'win32' ? '.exe' : '';
    // NOTE: The `./target/release` path is aligned with the path defined in .github/workflows/release_vscode.yml

    const releaseCandidate = join(context.extensionPath, `./target/release/oxc_language_server${ext}`);
    const debugCandidate = join(context.extensionPath, `./target/debug/oxc_language_server${ext}`);
    const envCandidate = process.env.SERVER_PATH_DEV;

    const candidates = [envCandidate, releaseCandidate, debugCandidate].filter(Boolean) as string[];
    for (const candidate of candidates) {
      try {
        await fsPromises.access(candidate);
        outputChannel.info(`Using detected oxc_language_server: ${candidate}`);
        return candidate;
      } catch {
        // continue
      }
    }

    outputChannel.error(
      `No oxc_language_server binary found. Tried: ${candidates.join(', ')}\n` +
      'Build one with: pnpm run server:build:release (or server:build:debug) in editors/vscode.'
    );
    // Return release path as last resort (will still fail fast, but message is logged)
    return releaseCandidate;
  }

  // External socket mode: if OXC_LS_CONNECT is set, connect instead of spawning.
  const externalSocketSpec = process.env.OXC_LS_CONNECT;
  let serverOptions: ServerOptions;
  if (externalSocketSpec) {
    const socketPath = externalSocketSpec.replace(/^unix:/, '');
    outputChannel.info(`Connecting to external oxc_language_server socket: ${socketPath}`);
    // Retry logic: attempt to connect several times with exponential backoff to avoid race condition.
    const maxAttempts = 8;
    const baseDelayMs = 75;
    serverOptions = () => new Promise<StreamInfo>((resolve, reject) => {
      let attempt = 0;
      const tryConnect = () => {
        attempt += 1;
        const socket = net.createConnection(socketPath, () => {
          outputChannel.info(`Connected to external language server after ${attempt} attempt(s).`);
          resolve({ reader: socket, writer: socket });
        });
        socket.on('error', (err) => {
          socket.destroy();
          if (attempt < maxAttempts) {
            const delay = baseDelayMs * (2 ** (attempt - 1));
            outputChannel.info(`Language server not ready (attempt ${attempt}/${maxAttempts}). Retrying in ${delay}ms...`);
            setTimeout(tryConnect, delay);
          } else {
            outputChannel.error(`Failed to connect to external language server after ${maxAttempts} attempts at ${socketPath}`, err);
            reject(err);
          }
        });
      };
      tryConnect();
    });
  } else {
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
    serverOptions = {
      run,
      debug: run,
    };
  }

  // see https://github.com/oxc-project/oxc/blob/9b475ad05b750f99762d63094174be6f6fc3c0eb/crates/oxc_linter/src/loader/partial_loader/mod.rs#L17-L20
  // This list is also sent to the language server to avoid hard-coded duplication of extensions
  // for workspace scanning & source file watchers.
  const supportedExtensions = ['astro', 'cjs', 'cts', 'js', 'jsx', 'mjs', 'mts', 'svelte', 'ts', 'tsx', 'vue'];

  // Helper to augment workspace configuration entries with supportedExtensions.
  type WorkspaceConfigEntry = (typeof configService.languageServerConfig)[number];
  const withSupportedExtensions = (workspaces: WorkspaceConfigEntry[]) =>
    workspaces.map((ws: WorkspaceConfigEntry) => ({
      ...ws,
      options: { ...ws.options, supportedExtensions },
    }));

  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [
      {
        pattern: `**/*.{${supportedExtensions.join(',')}}`,
        scheme: 'file',
      },
    ],
  initializationOptions: withSupportedExtensions(configService.languageServerConfig),
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

  context.subscriptions.push(onNotificationDispose);

  const onDeleteFilesDispose = workspace.onDidDeleteFiles((event) => {
    for (const fileUri of event.files) {
      client?.diagnostics?.delete(fileUri);
    }
  });

  context.subscriptions.push(onDeleteFilesDispose);

  const onDidChangeWorkspaceFoldersDispose = workspace.onDidChangeWorkspaceFolders(async (event) => {
    for (const folder of event.added) {
      configService.addWorkspaceConfig(folder);
    }
    for (const folder of event.removed) {
      configService.removeWorkspaceConfig(folder);
    }
  });

  context.subscriptions.push(onDidChangeWorkspaceFoldersDispose);

  configService.onConfigChange = async function onConfigChange(event) {
    updateStatsBar(context, this.vsCodeConfig.enable);

    if (client === undefined) {
      return;
    }

    // update the initializationOptions for a possible restart (keep them augmented)
    client.clientOptions.initializationOptions = withSupportedExtensions(this.languageServerConfig);

    if (configService.effectsWorkspaceConfigChange(event) && client.isRunning()) {
      await client.sendNotification('workspace/didChangeConfiguration', {
        settings: withSupportedExtensions(configService.languageServerConfig),
      });
    }
  };

  updateStatsBar(context, configService.vsCodeConfig.enable);
  if (allowedToStartServer) {
    if (configService.vsCodeConfig.enable) {
      await client.start();
    }
  } else {
    generateActivatorByConfig(configService.vsCodeConfig, context);
  }
}

export async function deactivate(): Promise<void> {
  if (!client) {
    return undefined;
  }
  await client.stop();
  client = undefined;
}

function updateStatsBar(context: ExtensionContext, enable: boolean) {
  if (!myStatusBarItem) {
    myStatusBarItem = window.createStatusBarItem(StatusBarAlignment.Right, 100);
    myStatusBarItem.command = OxcCommands.ToggleEnable;
    context.subscriptions.push(myStatusBarItem);
    myStatusBarItem.show();
  }
  let bgColor: string;
  let icon: string;
  if (!allowedToStartServer) {
    bgColor = 'statusBarItem.offlineBackground';
    icon = '$(circle-slash)';
  } else if (!enable) {
    bgColor = 'statusBarItem.warningBackground';
    icon = '$(check)';
  } else {
    bgColor = 'statusBarItem.activeBackground';
    icon = '$(check-all)';
  }

  myStatusBarItem.text = `${icon} oxc`;
  myStatusBarItem.backgroundColor = new ThemeColor(bgColor);
}

function generateActivatorByConfig(config: VSCodeConfig, context: ExtensionContext): void {
  const watcher = workspace.createFileSystemWatcher('**/.oxlintrc.json', false, true, !config.requireConfig);
  watcher.onDidCreate(async () => {
    allowedToStartServer = true;
    updateStatsBar(context, config.enable);
    if (client && !client.isRunning() && config.enable) {
      await client.start();
    }
  });

  watcher.onDidDelete(async () => {
    // only can be called when config.requireConfig
    allowedToStartServer = (await workspace.findFiles(`**/.oxlintrc.json`, '**/node_modules/**', 1)).length > 0;
    if (!allowedToStartServer) {
      updateStatsBar(context, false);
      if (client && client.isRunning()) {
        await client.stop();
      }
    }
  });

  context.subscriptions.push(watcher);
}
