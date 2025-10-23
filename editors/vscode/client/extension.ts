import { execFile } from 'node:child_process';
import { constants as fsConstants, promises as fsPromises } from 'node:fs';
import { homedir } from 'node:os';
import { delimiter, dirname, join } from 'node:path';
import { promisify } from 'node:util';

import {
  commands,
  ExtensionContext,
  LogOutputChannel,
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

const execFileAsync = promisify(execFile);
const nodeExecutableName = process.platform === 'win32' ? 'node.exe' : 'node';

type NodeEnvResolver = (
  env: NodeJS.ProcessEnv,
  outputChannel: LogOutputChannel,
) => Promise<NodeJS.ProcessEnv | null>;

function getPathKey(env: NodeJS.ProcessEnv): string {
  const existingKey = Object.keys(env).find(key => key.toLowerCase() === 'path');
  return existingKey ?? (process.platform === 'win32' ? 'Path' : 'PATH');
}

async function pathExists(candidate: string, mode: number = fsConstants.F_OK): Promise<boolean> {
  try {
    await fsPromises.access(candidate, mode);
    return true;
  } catch {
    return false;
  }
}

function prependPath(env: NodeJS.ProcessEnv, directory: string): NodeJS.ProcessEnv {
  const pathKey = getPathKey(env);
  const current = env[pathKey] ?? '';
  const entries = current.split(delimiter).filter(Boolean);
  if (!entries.includes(directory)) {
    entries.unshift(directory);
  }
  return { ...env, [pathKey]: entries.join(delimiter) };
}

async function hasNodeInPath(env: NodeJS.ProcessEnv): Promise<boolean> {
  const whichCommand = process.platform === 'win32' ? 'where' : 'which';
  try {
    const { stdout } = await execFileAsync(whichCommand, ['node'], { env });
    return stdout.trim().length > 0;
  } catch {
    return false;
  }
}

async function locateFnmExecutable(env: NodeJS.ProcessEnv): Promise<string | undefined> {
  const whichCommand = process.platform === 'win32' ? 'where' : 'which';
  try {
    await execFileAsync(whichCommand, ['fnm'], { env });
    return 'fnm';
  } catch {
    return undefined;
  }
}

function mergeFnmJsonEnv(
  jsonStr: string,
  env: NodeJS.ProcessEnv,
): NodeJS.ProcessEnv | null {
  try {
    const updates = JSON.parse(jsonStr) as Record<string, string>;
    let next = { ...env, ...updates };

    if (process.platform === 'win32' && updates.FNM_MULTISHELL_PATH) {
      next = prependPath(next, updates.FNM_MULTISHELL_PATH);
    } else if (updates.FNM_DIR) {
      next = prependPath(next, updates.FNM_DIR);
    }

    return next;
  } catch {
    return null;
  }
}

async function resolveFnmEnv(
  env: NodeJS.ProcessEnv,
  outputChannel: LogOutputChannel,
): Promise<NodeJS.ProcessEnv | null> {
  const fnmExecutable = await locateFnmExecutable(env);
  if (!fnmExecutable) {
    return null;
  }

  try {
    const { stdout } = await execFileAsync(fnmExecutable, ['env', '--json'], { env });
    const parsed = mergeFnmJsonEnv(stdout.toString(), env);
    if (parsed) {
      outputChannel.info('Type-aware linting detected Node.js via fnm.');
      return parsed;
    }
  } catch (error) {
    outputChannel.warn(`Failed to evaluate fnm environment: ${(error as Error).message}`);
  }

  return null;
}

async function resolveVoltaEnv(
  env: NodeJS.ProcessEnv,
  outputChannel: LogOutputChannel,
): Promise<NodeJS.ProcessEnv | null> {
  const voltaHomes = new Set<string>();
  const envHome = env.VOLTA_HOME ?? process.env.VOLTA_HOME;
  if (envHome) {
    voltaHomes.add(envHome);
  }

  const home = homedir();
  if (home) {
    voltaHomes.add(join(home, '.volta'));
  }

  const localAppData = env.LOCALAPPDATA ?? process.env.LOCALAPPDATA;
  if (localAppData) {
    voltaHomes.add(join(localAppData, 'Volta'));
  }

  const appData = env.APPDATA ?? process.env.APPDATA;
  if (appData) {
    voltaHomes.add(join(appData, 'Volta'));
  }

  for (const voltaHome of voltaHomes) {
    if (!voltaHome) {
      continue;
    }
    const binDir = join(voltaHome, 'bin');
    const nodePath = join(binDir, nodeExecutableName);
    if (await pathExists(nodePath)) {
      outputChannel.info(`Type-aware linting detected Node.js via Volta at ${binDir}.`);
      return prependPath(env, binDir);
    }
  }

  return null;
}

async function resolveNvmEnv(
  env: NodeJS.ProcessEnv,
  outputChannel: LogOutputChannel,
): Promise<NodeJS.ProcessEnv | null> {
  const nvmSymlink = env.NVM_SYMLINK ?? process.env.NVM_SYMLINK;
  if (nvmSymlink) {
    const nodePath = join(nvmSymlink, nodeExecutableName);
    if (await pathExists(nodePath)) {
      outputChannel.info(`Type-aware linting detected Node.js via nvm-windows at ${nvmSymlink}.`);
      return prependPath(env, nvmSymlink);
    }
  }

  const nvmDir = env.NVM_DIR ?? process.env.NVM_DIR ?? (homedir() ? join(homedir(), '.nvm') : undefined);
  if (!nvmDir) {
    return null;
  }

  const nvmScript = join(nvmDir, 'nvm.sh');
  if (await pathExists(nvmScript)) {
    try {
      const { stdout } = await execFileAsync('bash', ['-lc', `. "${nvmScript}" >/dev/null 2>&1 && nvm which default`], {
        env,
      });
      const nodePath = stdout.toString().trim();
      if (nodePath) {
        const binDir = dirname(nodePath);
        if (await pathExists(nodePath)) {
          outputChannel.info(`Type-aware linting detected Node.js via nvm at ${binDir}.`);
          return prependPath(env, binDir);
        }
      }
    } catch {
      // fall back to inspecting the alias file below
    }
  }

  const aliasDefault = join(nvmDir, 'alias', 'default');
  try {
    const aliasContent = await fsPromises.readFile(aliasDefault, 'utf8');
    const version = aliasContent.split(/\s+/).filter(Boolean)[0]?.trim();
    if (!version || version === 'system' || version.startsWith('lts/')) {
      return null;
    }
    const sanitized = version.startsWith('v') ? version.slice(1) : version;
    // The following path construction assumes the standard Unix-like nvm directory structure :
    // $NVM_DIR/versions/node/<version>/bin
    // nvm-windows is handled separately above via NVM_SYMLINK
    const binDir = join(nvmDir, 'versions', 'node', sanitized, 'bin');
    const nodePath = join(binDir, nodeExecutableName);
    if (await pathExists(nodePath)) {
      outputChannel.info(`Type-aware linting detected Node.js via nvm default alias at ${binDir}.`);
      return prependPath(env, binDir);
    }
  } catch {
    // ignore
  }

  return null;
}

async function ensureNodeOnPath(env: NodeJS.ProcessEnv, outputChannel: LogOutputChannel): Promise<NodeJS.ProcessEnv> {
  if (await hasNodeInPath(env)) {
    return env;
  }

  const resolvers: NodeEnvResolver[] = [resolveVoltaEnv, resolveFnmEnv, resolveNvmEnv];

  for (const resolver of resolvers) {
    const resolved = await resolver(env, outputChannel);
    if (!resolved) {
      continue;
    }
    if (await hasNodeInPath(resolved)) {
      return resolved;
    }
    env = resolved;
  }

  outputChannel.warn('Type-aware linting is enabled but Node.js could not be located automatically.');
  return env;
}

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

  async function findBinary(): Promise<string> {
    let bin = configService.getUserServerBinPath();
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

  const command = await findBinary();
  const typeAwareEnabled = configService.languageServerConfig.some(({ options }) => options.typeAware);

  let runEnv: NodeJS.ProcessEnv = {
    ...process.env,
    RUST_LOG: process.env.RUST_LOG || 'info',
  };

  if (typeAwareEnabled) {
    runEnv = await ensureNodeOnPath(runEnv, outputChannel);
  }

  const run: Executable = {
    command: command!,
    options: {
      env: runEnv,
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
