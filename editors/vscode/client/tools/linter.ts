import { promises as fsPromises } from "node:fs";

import {
  commands,
  ConfigurationChangeEvent,
  ExtensionContext,
  LogOutputChannel,
  Uri,
  window,
  workspace,
} from "vscode";

import {
  ConfigurationParams,
  ExecuteCommandRequest,
  ShowMessageNotification,
} from "vscode-languageclient";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

import { OxcCommands } from "../commands";
import { ConfigService } from "../ConfigService";
import StatusBarItemHandler from "../StatusBarItemHandler";
import { VSCodeConfig } from "../VSCodeConfig";
import { onClientNotification, runExecutable } from "./lsp_helper";
import ToolInterface from "./ToolInterface";

const languageClientName = "oxc";

const enum LspCommands {
  FixAll = "oxc.fixAll",
}

export default class LinterTool implements ToolInterface {
  // Global flag to check if the user allows us to start the server.
  // When `oxc.requireConfig` is `true`, make sure one oxlint config file is present.
  private allowedToStartServer: boolean = false;

  // LSP client instance
  private client: LanguageClient | undefined;

  async getBinary(
    outputChannel: LogOutputChannel,
    configService: ConfigService,
  ): Promise<string | undefined> {
    if (process.env.SERVER_PATH_DEV) {
      return process.env.SERVER_PATH_DEV;
    }
    const bin = await configService.getOxlintServerBinPath();
    if (bin) {
      try {
        await fsPromises.access(bin);
        return bin;
      } catch (e) {
        outputChannel.error(`Invalid bin path: ${bin}`, e);
      }
    }
  }

  async activate(
    context: ExtensionContext,
    outputChannel: LogOutputChannel,
    configService: ConfigService,
    statusBarItemHandler: StatusBarItemHandler,
    binaryPath?: string,
  ): Promise<void> {
    if (!binaryPath) {
      statusBarItemHandler.updateTool("linter", false, "No valid oxlint binary found.");
      outputChannel.appendLine("No valid oxlint binary found. Linter will not be activated.");
      return Promise.resolve();
    }

    this.allowedToStartServer = configService.vsCodeConfig.requireConfig
      ? await hasOxlintConfig()
      : true;

    const restartCommand = commands.registerCommand(OxcCommands.RestartServerLint, async () => {
      await this.restartClient();
      this.updateStatusBar(statusBarItemHandler, configService.vsCodeConfig.enable);
    });

    const toggleEnable = commands.registerCommand(OxcCommands.ToggleEnableLint, async () => {
      await configService.vsCodeConfig.updateEnable(!configService.vsCodeConfig.enable);

      await this.toggleClient(configService);
      this.updateStatusBar(statusBarItemHandler, configService.vsCodeConfig.enable);
    });

    const applyAllFixesFile = commands.registerCommand(OxcCommands.ApplyAllFixesFile, async () => {
      if (!this.client) {
        window.showErrorMessage("oxc client not found");
        return;
      }
      const textEditor = window.activeTextEditor;
      if (!textEditor) {
        window.showErrorMessage("active text editor not found");
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

      await this.client.sendRequest(ExecuteCommandRequest.type, params);
    });

    context.subscriptions.push(restartCommand, toggleEnable, applyAllFixesFile);

    const run: Executable = runExecutable(
      binaryPath,
      "oxlint",
      configService.vsCodeConfig.nodePath,
      configService.vsCodeConfig.binPathTsGoLint,
    );
    const serverOptions: ServerOptions = {
      run,
      debug: run,
    };

    outputChannel.info(`Using server binary at: ${binaryPath}`);

    // see https://github.com/oxc-project/oxc/blob/9b475ad05b750f99762d63094174be6f6fc3c0eb/crates/oxc_linter/src/loader/partial_loader/mod.rs#L17-L20
    const supportedExtensions = [
      "astro",
      "cjs",
      "cts",
      "js",
      "jsx",
      "mjs",
      "mts",
      "svelte",
      "ts",
      "tsx",
      "vue",
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
      ],
      initializationOptions: configService.oxlintServerConfig,
      outputChannel,
      traceOutputChannel: outputChannel,
      diagnosticPullOptions: {
        onChange: true,
        onSave: true,
        onTabs: false,
        filter: (document, mode) => !configService.shouldRequestDiagnostics(document.uri, mode),
      },
      middleware: {
        handleDiagnostics: (uri, diagnostics, next) => {
          for (const diag of diagnostics) {
            // https://github.com/oxc-project/oxc/issues/12404
            if (
              typeof diag.code === "object" &&
              diag.code?.value === "eslint-plugin-unicorn(filename-case)"
            ) {
              diag.message +=
                "\nYou may need to close the file and restart VSCode after renaming a file by only casing.";
            }
          }
          next(uri, diagnostics);
        },
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
                configService.getWorkspaceConfig(Uri.parse(item.scopeUri))?.toOxlintConfig() ?? null
              );
            });
          },
        },
      },
    };

    this.client = new LanguageClient(languageClientName, serverOptions, clientOptions);

    const onNotificationDispose = this.client.onNotification(
      ShowMessageNotification.type,
      (params) => {
        onClientNotification(params, outputChannel);
      },
    );

    context.subscriptions.push(onNotificationDispose);

    const onDeleteFilesDispose = workspace.onDidDeleteFiles((event) => {
      for (const fileUri of event.files) {
        this.client?.diagnostics?.delete(fileUri);
      }
    });

    context.subscriptions.push(onDeleteFilesDispose);

    if (this.allowedToStartServer) {
      if (configService.vsCodeConfig.enable) {
        await this.client.start();
      }
    } else {
      this.generateActivatorByConfig(configService.vsCodeConfig, context, statusBarItemHandler);
    }

    this.updateStatusBar(statusBarItemHandler, configService.vsCodeConfig.enable);
  }

  async deactivate(): Promise<void> {
    if (!this.client) {
      return undefined;
    }
    await this.client.stop();
    this.client = undefined;
  }

  async toggleClient(configService: ConfigService): Promise<void> {
    if (this.client === undefined || !this.allowedToStartServer) {
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

  async restartClient(): Promise<void> {
    if (this.client === undefined) {
      window.showErrorMessage("oxlint client not found");
      return;
    }

    try {
      if (this.client.isRunning()) {
        await this.client.restart();
        window.showInformationMessage("oxlint server restarted.");
      } else {
        await this.client.start();
      }
    } catch (err) {
      this.client.error("Restarting oxlint client failed", err, "force");
    }
  }

  async onConfigChange(
    event: ConfigurationChangeEvent,
    configService: ConfigService,
    statusBarItemHandler: StatusBarItemHandler,
  ): Promise<void> {
    if (event.affectsConfiguration(`${ConfigService.namespace}.enable`)) {
      await this.toggleClient(configService); // update the client state
    }
    this.updateStatusBar(statusBarItemHandler, configService.vsCodeConfig.enable);

    if (this.client === undefined) {
      return;
    }

    // update the initializationOptions for a possible restart
    this.client.clientOptions.initializationOptions = configService.oxlintServerConfig;

    if (configService.effectsWorkspaceConfigChange(event) && this.client.isRunning()) {
      await this.client.sendNotification("workspace/didChangeConfiguration", {
        settings: configService.oxlintServerConfig,
      });
    }
  }

  /**
   * ------- Helpers -------
   */

  /**
   * Get the status bar state based on whether oxc is enabled and allowed to start.
   */
  getStatusBarState(enable: boolean): {
    isEnabled: boolean;
    tooltipText?: string;
  } {
    if (!this.allowedToStartServer) {
      return {
        isEnabled: false,
        tooltipText: "no oxlint config found",
      };
    } else if (!enable) {
      return {
        isEnabled: false,
        tooltipText: "`oxc.enable` is false",
      };
    }

    return {
      isEnabled: true,
    };
  }

  updateStatusBar(statusBarItemHandler: StatusBarItemHandler, enable: boolean) {
    const { isEnabled, tooltipText } = this.getStatusBarState(enable);

    let text =
      `[$(terminal) Open Output](command:${OxcCommands.ShowOutputChannelLint})\n\n` +
      `[$(refresh) Restart Server](command:${OxcCommands.RestartServerLint})\n\n`;

    if (enable) {
      text += `[$(stop) Stop Server](command:${OxcCommands.ToggleEnableLint})\n\n`;
    } else {
      text += `[$(play) Start Server](command:${OxcCommands.ToggleEnableLint})\n\n`;
    }

    if (tooltipText) {
      text = `${tooltipText}\n\n` + text;
    }

    statusBarItemHandler.updateTool(
      "linter",
      isEnabled,
      text,
      this.client?.initializeResult?.serverInfo?.version,
    );
  }

  generateActivatorByConfig(
    config: VSCodeConfig,
    context: ExtensionContext,
    statusBarItemHandler: StatusBarItemHandler,
  ): void {
    const watchers = OXLINT_CONFIG_GLOBS.map((glob) =>
      workspace.createFileSystemWatcher(glob, false, true, !config.requireConfig),
    );

    for (const watcher of watchers) {
      watcher.onDidCreate(async () => {
        this.allowedToStartServer = true;
        this.updateStatusBar(statusBarItemHandler, config.enable);
        if (this.client && !this.client.isRunning() && config.enable) {
          await this.client.start();
        }
      });

      watcher.onDidDelete(async () => {
        // only can be called when config.requireConfig
        this.allowedToStartServer = await hasOxlintConfig();
        if (!this.allowedToStartServer) {
          this.updateStatusBar(statusBarItemHandler, false);
          if (this.client && this.client.isRunning()) {
            await this.client.stop();
          }
        }
      });

      context.subscriptions.push(watcher);
    }
  }
}

const OXLINT_CONFIG_GLOBS = ["**/.oxlintrc.json", "**/oxlint.config.ts"];

async function hasOxlintConfig(): Promise<boolean> {
  const configs = await Promise.all(
    OXLINT_CONFIG_GLOBS.map((glob) => workspace.findFiles(glob, "**/node_modules/**", 1)),
  );
  return configs.some((matches) => matches.length > 0);
}
