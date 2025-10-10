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
} from 'vscode-languageclient';

import { Executable, LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

import { join } from 'node:path';
import { ConfigService } from './ConfigService';
import { VSCodeConfig } from './VSCodeConfig';

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

// Global flag to check if the user allows us to start the server.
// When `oxc.requireConfig` is `true`, make sure one `.oxlintrc.json` file is present.
let allowedToStartServer: boolean;

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();
  allowedToStartServer = configService.vsCodeConfig.requireConfig
    ? (await workspace.findFiles(`**/.oxlintrc.json`, '**/node_modules/**', 1)).length > 0
    : true;

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

  const outputChannel = window.createOutputChannel(outputChannelName, { log: true });

  context.subscriptions.push(
    applyAllFixesFile,
    restartCommand,
    showOutputCommand,
    toggleEnable,
    configService,
    outputChannel,
  );

  async function findBinary(bin: string | undefined): Promise<string> {
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
    return (
      process.env.SERVER_PATH_DEV ??
        join(context.extensionPath, `./target/release/oxc_language_server${ext}`)
    );
  }

  const userDefinedBinary = configService.getUserServerBinPath();
  const command = await findBinary(userDefinedBinary);
  const run: Executable = {
    command: command!,
    options: {
      // On Windows we need to run the binary in a shell to be able to execute the shell npm bin script.
      // This is only needed when the user explicitly configures the binary to point to the npm bin script.
      // The extension is shipped with the `.exe` file, we don't need to run it in a shell.
      // Searching for the right `.exe` file inside `node_modules/` is not reliable as it depends on
      // the package manager used (npm, yarn, pnpm, etc) and the package version.
      // The npm bin script is a shell script that points to the actual binary.
      // Security: We validated the userDefinedBinary in `configService.getUserServerBinPath()`.
      shell: process.platform === 'win32' && command === userDefinedBinary &&
        userDefinedBinary?.endsWith('node_modules\\.bin\\oxc_language_server'),
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

  // see https://github.com/oxc-project/oxc/blob/9b475ad05b750f99762d63094174be6f6fc3c0eb/crates/oxc_linter/src/loader/partial_loader/mod.rs#L17-L20
  const supportedExtensions = ['astro', 'cjs', 'cts', 'js', 'jsx', 'mjs', 'mts', 'svelte', 'ts', 'tsx', 'vue'];

  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{
      pattern: `**/*.{${supportedExtensions.join(',')}}`,
      scheme: 'file',
    }],
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
          return params.items.map(item => {
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
  client = new LanguageClient(
    languageClientName,
    serverOptions,
    clientOptions,
  );

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

    // update the initializationOptions for a possible restart
    client.clientOptions.initializationOptions = this.languageServerConfig;

    if (configService.effectsWorkspaceConfigChange(event) && client.isRunning()) {
      await client.sendNotification(
        'workspace/didChangeConfiguration',
        {
          settings: this.languageServerConfig,
        },
      );
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

function updateStatsBar(
  context: ExtensionContext,
  enable: boolean,
) {
  if (!myStatusBarItem) {
    myStatusBarItem = window.createStatusBarItem(
      StatusBarAlignment.Right,
      100,
    );
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
  const watcher = workspace.createFileSystemWatcher('**/.oxlintrc.json', false, true, true);
  watcher.onDidCreate(async () => {
    watcher.dispose();
    allowedToStartServer = true;
    updateStatsBar(context, config.enable);
    if (client && !client.isRunning() && config.enable) {
      await client.start();
    }
  });

  context.subscriptions.push(watcher);
}
