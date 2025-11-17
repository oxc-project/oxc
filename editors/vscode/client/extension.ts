import { commands, ExtensionContext, window, workspace } from 'vscode';

import { OxcCommands } from './commands';
import { ConfigService } from './ConfigService';
import StatusBarItemHandler from './StatusBarItemHandler';
import Formatter from './tools/formatter';
import Linter from './tools/linter';
import ToolInterface from './tools/ToolInterface';

const outputChannelName = 'Oxc';
const tools: ToolInterface[] = [];

if (process.env.SKIP_LINTER_TEST !== 'true') {
  tools.push(new Linter());
}
if (process.env.SKIP_FORMATTER_TEST !== 'true') {
  tools.push(new Formatter());
}

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();

  const outputChannelLint = window.createOutputChannel(outputChannelName + ' (Lint)', {
    log: true,
  });

  const outputChannelFormat = window.createOutputChannel(outputChannelName + ' (Fmt)', {
    log: true,
  });

  const restartCommand = commands.registerCommand(OxcCommands.RestartServer, async () => {
    for (const tool of tools) {
      await tool.restartClient();
    }
  });

  const showOutputCommand = commands.registerCommand(OxcCommands.ShowOutputChannel, () => {
    outputChannelLint.show();
  });

  const toggleEnable = commands.registerCommand(OxcCommands.ToggleEnable, async () => {
    await configService.vsCodeConfig.updateEnable(!configService.vsCodeConfig.enable);

    for (const tool of tools) {
      await tool.toggleClient(configService);
    }
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
    outputChannelLint,
    outputChannelFormat,
    onDidChangeWorkspaceFoldersDispose,
    statusBarItemHandler,
  );

  configService.onConfigChange = async function onConfigChange(event) {
    for (const tool of tools) {
      await tool.onConfigChange(event, configService, statusBarItemHandler);
    }
  };

  for (const tool of tools) {
    await tool.activate(
      context,
      tool instanceof Linter ? outputChannelLint : outputChannelFormat,
      configService,
      statusBarItemHandler,
    );
  }
}

export async function deactivate(): Promise<void> {
  for (const tool of tools) {
    await tool.deactivate();
  }
}
