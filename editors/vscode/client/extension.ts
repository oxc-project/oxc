import { commands, ExtensionContext, window, workspace } from "vscode";

import { OxcCommands } from "./commands";
import { ConfigService } from "./ConfigService";
import StatusBarItemHandler from "./StatusBarItemHandler";
import Linter from "./tools/linter";

const outputChannelName = "Oxc";
const linter = new Linter();

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();

  const outputChannel = window.createOutputChannel(outputChannelName, {
    log: true,
  });

  const showOutputCommand = commands.registerCommand(OxcCommands.ShowOutputChannel, () => {
    outputChannel.show();
  });

  const onDidChangeWorkspaceFoldersDispose = workspace.onDidChangeWorkspaceFolders(
    async (event) => {
      for (const folder of event.added) {
        configService.addWorkspaceConfig(folder);
      }
      for (const folder of event.removed) {
        configService.removeWorkspaceConfig(folder);
      }
    },
  );

  const statusBarItemHandler = new StatusBarItemHandler(context.extension.packageJSON?.version);

  context.subscriptions.push(
    showOutputCommand,
    configService,
    outputChannel,
    onDidChangeWorkspaceFoldersDispose,
    statusBarItemHandler,
  );

  configService.onConfigChange = async function onConfigChange(event) {
    await linter.onConfigChange(event, configService, statusBarItemHandler);
  };
  const binaryPath = await linter.getBinary(context, outputChannel, configService);

  // For the linter this should never happen, but just in case.
  if (!binaryPath) {
    statusBarItemHandler.setColorAndIcon("statusBarItem.errorBackground", "error");
    statusBarItemHandler.updateToolTooltip(
      "linter",
      "Error: No valid oxc language server binary found.",
    );
    statusBarItemHandler.show();
    outputChannel.error("No valid oxc language server binary found.");
    return;
  }

  await linter.activate(context, binaryPath, outputChannel, configService, statusBarItemHandler);
  // Show status bar item after activation
  statusBarItemHandler.show();
}

export async function deactivate(): Promise<void> {
  await linter.deactivate();
}
