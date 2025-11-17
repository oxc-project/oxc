import { promises as fsPromises } from 'node:fs';

import { ConfigurationChangeEvent, ExtensionContext, LogOutputChannel, Uri, window } from 'vscode';

import { ConfigurationParams, ShowMessageNotification } from 'vscode-languageclient';

import { Executable, LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

import { ConfigService } from '../ConfigService';
import StatusBarItemHandler from '../StatusBarItemHandler';
import { onClientNotification, runExecutable } from './lsp_helper';
import ToolInterface from './ToolInterface';

const languageClientName = 'oxc';

export default class FormatterTool implements ToolInterface {
  // LSP client instance
  private client: LanguageClient | undefined;

  async activate(context: ExtensionContext, outputChannel: LogOutputChannel, configService: ConfigService) {
    async function findBinary(): Promise<string | undefined> {
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

    const path = await findBinary();

    if (!path) {
      outputChannel.error('oxfmt server binary not found.');
      return;
    }

    outputChannel.info(`Using server binary at: ${path}`);

    const run: Executable = runExecutable(path, configService.vsCodeConfig.nodePath);

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
    this.client = new LanguageClient(languageClientName, serverOptions, clientOptions);

    const onNotificationDispose = this.client.onNotification(ShowMessageNotification.type, (params) => {
      onClientNotification(params, outputChannel);
    });

    context.subscriptions.push(onNotificationDispose);

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
      window.showErrorMessage('oxc client not found');
      return;
    }

    try {
      if (this.client.isRunning()) {
        await this.client.restart();
        window.showInformationMessage('oxc server restarted.');
      } else {
        await this.client.start();
      }
    } catch (err) {
      this.client.error('Restarting client failed', err, 'force');
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
    _statusBarItemHandler: StatusBarItemHandler,
  ): Promise<void> {
    updateStatsBar(configService.vsCodeConfig.enable);

    if (this.client === undefined) {
      return;
    }

    // update the initializationOptions for a possible restart
    this.client.clientOptions.initializationOptions = configService.languageServerConfig;

    if (configService.effectsWorkspaceConfigChange(event) && this.client.isRunning()) {
      await this.client.sendNotification('workspace/didChangeConfiguration', {
        settings: configService.languageServerConfig,
      });
    }
  }
}

function updateStatsBar(_enable: boolean) {
  // TODO: implement formatter status tooltip and handle color and icon
}
