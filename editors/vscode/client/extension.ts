import { commands, ExtensionContext, window, workspace } from "vscode";

import { OxcCommands } from "./commands";
import { ConfigService } from "./ConfigService";
import StatusBarItemHandler from "./StatusBarItemHandler";
import Formatter from "./tools/formatter";
import Linter from "./tools/linter";
import ToolInterface from "./tools/ToolInterface";

const outputChannelName = "Oxc";
const tools: ToolInterface[] = [];

if (process.env.SKIP_LINTER_TEST !== "true") {
  tools.push(new Linter());
}
if (process.env.SKIP_FORMATTER_TEST !== "true") {
  tools.push(new Formatter());
}

export async function activate(context: ExtensionContext) {
  const configService = new ConfigService();

  const outputChannelLint = window.createOutputChannel(outputChannelName + " (Lint)", {
    log: true,
  });

  const outputChannelFormat = window.createOutputChannel(outputChannelName + " (Fmt)", {
    log: true,
  });

  const showOutputLintCommand = commands.registerCommand(OxcCommands.ShowOutputChannelLint, () => {
    outputChannelLint.show();
  });

  const showOutputFmtCommand = commands.registerCommand(OxcCommands.ShowOutputChannelFmt, () => {
    outputChannelFormat.show();
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
    showOutputLintCommand,
    showOutputFmtCommand,
    configService,
    outputChannelLint,
    outputChannelFormat,
    onDidChangeWorkspaceFoldersDispose,
    statusBarItemHandler,
  );

  configService.onConfigChange = async function onConfigChange(event) {
    await Promise.all(
      tools.map((tool) => tool.onConfigChange(event, configService, statusBarItemHandler)),
    );
  };

  const binaryPaths = await Promise.all(
    tools.map((tool) =>
      tool.getBinary(
        context,
        tool instanceof Linter ? outputChannelLint : outputChannelFormat,
        configService,
      ),
    ),
  );

  await Promise.all(
    tools.map((tool): Promise<void> => {
      const binaryPath = binaryPaths[tools.indexOf(tool)];

      if (!binaryPath && tool instanceof Linter) {
        statusBarItemHandler.setColorAndIcon("statusBarItem.errorBackground", "error");
        statusBarItemHandler.updateToolTooltip(
          "linter",
          "**oxlint disabled**\n\nError: No valid oxlint binary found.",
        );
        return Promise.resolve();
      }

      if (!binaryPath && tool instanceof Formatter) {
        // No valid binary found for the formatter.
        statusBarItemHandler.updateToolTooltip(
          "formatter",
          "**oxfmt disabled**\n\nNo valid oxfmt binary found.",
        );
        outputChannelFormat.appendLine(
          "No valid oxfmt binary found. Formatter will not be activated.",
        );
        return Promise.resolve();
      }

      // binaryPath is guaranteed to be defined here.
      const binaryPathResolved = binaryPath!;

      return tool.activate(
        context,
        binaryPathResolved,
        tool instanceof Linter ? outputChannelLint : outputChannelFormat,
        configService,
        statusBarItemHandler,
      );
    }),
  );

  // Finally show the status bar item.
  statusBarItemHandler.show();
}

export async function deactivate(): Promise<void> {
  await Promise.all(tools.map((tool) => tool.deactivate()));
}
