import { promises as fsPromises } from "node:fs";

import {
  commands,
  ConfigurationChangeEvent,
  ExtensionContext,
  LogOutputChannel,
  Uri,
  window,
} from "vscode";

import { ConfigurationParams, ShowMessageNotification } from "vscode-languageclient";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

import { OxcCommands } from "../commands";
import { ConfigService } from "../ConfigService";
import StatusBarItemHandler from "../StatusBarItemHandler";
import { onClientNotification, runExecutable } from "./lsp_helper";
import ToolInterface from "./ToolInterface";

const languageClientName = "oxc";

export default class FormatterTool implements ToolInterface {
  // LSP client instance
  private client: LanguageClient | undefined;

  async getBinary(
    _context: ExtensionContext,
    outputChannel: LogOutputChannel,
    configService: ConfigService,
  ): Promise<string | undefined> {
    const bin = await configService.getOxfmtServerBinPath();
    if (bin) {
      try {
        await fsPromises.access(bin);
        return bin;
      } catch (e) {
        outputChannel.error(`Invalid bin path: ${bin}`, e);
      }
    }
    return process.env.SERVER_PATH_DEV;
  }

  async activate(
    context: ExtensionContext,
    binaryPath: string,
    outputChannel: LogOutputChannel,
    configService: ConfigService,
    statusBarItemHandler: StatusBarItemHandler,
  ) {
    const restartCommand = commands.registerCommand(OxcCommands.RestartServerFmt, async () => {
      await this.restartClient();
    });

    outputChannel.info(`Using server binary at: ${binaryPath}`);

    const run: Executable = runExecutable(binaryPath, configService.vsCodeConfig.nodePath);

    const serverOptions: ServerOptions = {
      run,
      debug: run,
    };

    // This list is not used as-is for implementation to determine whether formatting processing is possible.
    const supportedExtensions = [
      "cjs",
      "cts",
      "js",
      "jsx",
      "mjs",
      "mts",
      "ts",
      "tsx",
      // https://github.com/oxc-project/oxc/blob/f3e9913f534e36195b9b5a6244dd21076ed8715e/crates/oxc_formatter/src/service/parse_utils.rs#L24-L45
      "_js",
      "bones",
      "es",
      "es6",
      "gs",
      "jake",
      "javascript",
      "jsb",
      "jscad",
      "jsfl",
      "jslib",
      "jsm",
      "jspre",
      "jss",
      "njs",
      "pac",
      "sjs",
      "ssjs",
      "xsjs",
      "xsjslib",
      // https://github.com/oxc-project/oxc/blob/f3e9913f534e36195b9b5a6244dd21076ed8715e/crates/oxc_formatter/src/service/parse_utils.rs#L73
      // allow `*.start.frag` and `*.end.frag`,
      "frag",
    ];

    // Special filenames that are valid JS files
    // https://github.com/oxc-project/oxc/blob/f3e9913f534e36195b9b5a6244dd21076ed8715e/crates/oxc_formatter/src/service/parse_utils.rs#L47C4-L52
    const specialFilenames = [
      "Jakefile",

      // covered by the "frag" extension above
      // "start.frag",
      // "end.frag",
    ];

    // If the extension is launched in debug mode then the debug server options are used
    // Otherwise the run options are used
    // Options to control the language client
    const clientOptions: LanguageClientOptions = {
      // Register the server for plain text documents
      documentSelector: [
        {
          pattern: `**/*.{${supportedExtensions.join(",")}}`,
          scheme: "file",
        },
        ...specialFilenames.map((filename) => ({
          pattern: `**/${filename}`,
          scheme: "file",
        })),
      ],
      initializationOptions: configService.formatterServerConfig,
      outputChannel,
      traceOutputChannel: outputChannel,
      middleware: {
        workspace: {
          configuration: (params: ConfigurationParams) => {
            return params.items.map((item) => {
              if (item.section !== "oxc_language_server") {
                return null;
              }
              if (item.scopeUri === undefined) {
                return null;
              }

              return (
                configService.getWorkspaceConfig(Uri.parse(item.scopeUri))?.toOxfmtConfig() ?? null
              );
            });
          },
        },
      },
    };

    // Create the language client and start the client.
    this.client = new LanguageClient(languageClientName, serverOptions, clientOptions);

    const onNotificationDispose = this.client.onNotification(
      ShowMessageNotification.type,
      (params) => {
        onClientNotification(params, outputChannel);
      },
    );

    context.subscriptions.push(restartCommand, onNotificationDispose);

    updateStatsBar(statusBarItemHandler, configService);
    if (configService.vsCodeConfig.enable) {
      await this.client.start();
    }
  }

  async deactivate(): Promise<void> {
    if (!this.client) {
      return undefined;
    }
    await this.client.stop();
    this.client = undefined;
  }

  async restartClient(): Promise<void> {
    if (this.client === undefined) {
      window.showErrorMessage("oxc client not found");
      return;
    }

    try {
      if (this.client.isRunning()) {
        await this.client.restart();
        window.showInformationMessage("oxc server restarted.");
      } else {
        await this.client.start();
      }
    } catch (err) {
      this.client.error("Restarting client failed", err, "force");
    }
  }

  async toggleClient(configService: ConfigService): Promise<void> {
    if (this.client === undefined) {
      return;
    }

    if (this.client.isRunning()) {
      if (!configService.vsCodeConfig.enable) {
        await this.client.stop();
      }
    } else {
      if (configService.vsCodeConfig.enable) {
        await this.client.start();
      }
    }
  }

  async onConfigChange(
    event: ConfigurationChangeEvent,
    configService: ConfigService,
    statusBarItemHandler: StatusBarItemHandler,
  ): Promise<void> {
    updateStatsBar(statusBarItemHandler, configService);

    if (this.client === undefined) {
      return;
    }

    // update the initializationOptions for a possible restart
    this.client.clientOptions.initializationOptions = configService.formatterServerConfig;

    if (configService.effectsWorkspaceConfigChange(event) && this.client.isRunning()) {
      await this.client.sendNotification("workspace/didChangeConfiguration", {
        settings: configService.formatterServerConfig,
      });
    }
  }
}

function updateStatsBar(statusBarItemHandler: StatusBarItemHandler, configService: ConfigService) {
  let text = configService.vsCodeConfig.enable ? `**oxfmt enabled**\n\n` : `**oxfmt disabled**\n\n`;

  text +=
    `[$(terminal) Open Output](command:${OxcCommands.ShowOutputChannelFmt})\n\n` +
    `[$(refresh) Restart Server](command:${OxcCommands.RestartServerFmt})\n\n`;

  statusBarItemHandler.updateToolTooltip("formatter", text);
}
