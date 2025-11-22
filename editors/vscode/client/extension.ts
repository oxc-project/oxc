import { commands, ExtensionContext, window, workspace } from 'vscode';

import { OxcCommands } from './commands';
import { ConfigService } from './ConfigService';
import {
  activate as activateLinter,
  deactivate as deactivateLinter,
  onConfigChange as onConfigChangeLinter,
  restartClient,
  toggleClient,
} from './linter';
import StatusBarItemHandler from './StatusBarItemHandler';

const outputChannelName = 'Oxc';

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();

  const outputChannel = window.createOutputChannel(outputChannelName, {
    log: true,
  });

  const restartCommand = commands.registerCommand(OxcCommands.RestartServer, async () => {
    await restartClient();
  });

  const showOutputCommand = commands.registerCommand(OxcCommands.ShowOutputChannel, () => {
    outputChannel.show();
  });

  const toggleEnable = commands.registerCommand(OxcCommands.ToggleEnable, async () => {
    await configService.vsCodeConfig.updateEnable(!configService.vsCodeConfig.enable);

    await toggleClient(configService);
  });

  const onDidChangeWorkspaceFoldersDispose = workspace.onDidChangeWorkspaceFolders(async (event) => {
    for (const folder of event.added) {
      configService.addWorkspaceConfig(folder);
    }
    for (const folder of event.removed) {
      configService.removeWorkspaceConfig(folder);
    }
  });

  const statusBarItemHandler = new StatusBarItemHandler(context.extension.packageJSON?.version);

  context.subscriptions.push(
    restartCommand,
    showOutputCommand,
    toggleEnable,
    configService,
    outputChannel,
    onDidChangeWorkspaceFoldersDispose,
    statusBarItemHandler,
  );

  configService.onConfigChange = async function onConfigChange(event) {
    await onConfigChangeLinter(event, configService, statusBarItemHandler);
  };

  await activateLinter(context, outputChannel, configService, statusBarItemHandler);
  // Show status bar item after activation
  statusBarItemHandler.show();
}

export async function deactivate(): Promise<void> {
  await deactivateLinter();
}
