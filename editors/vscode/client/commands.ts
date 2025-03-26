import { commands, window } from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';

import { ExecuteCommandRequest } from 'vscode-languageclient';
import { Config } from './Config';
import { findOxlintrcConfigFiles, sendDidChangeWatchedFilesNotificationWith, startClient } from './utilities';

const commandPrefix = 'oxc';

export const enum OxcCommands {
  RestartServer = `${commandPrefix}.restartServer`,
  ApplyAllFixesFile = `${commandPrefix}.applyAllFixesFile`,
  ShowOutputChannel = `${commandPrefix}.showOutputChannel`,
  ToggleEnable = `${commandPrefix}.toggleEnable`,
}

const enum LspCommands {
  FixAll = 'oxc.fixAll',
}

// we need a own function to get the updated reference in the callback
// or else we always get undefined
export type LanguageClientGetter = () => LanguageClient;

export const restartCommand = (clientGetter: LanguageClientGetter) =>
  commands.registerCommand(
    OxcCommands.RestartServer,
    async () => {
      const client = clientGetter();
      if (!client) {
        window.showErrorMessage('oxc client not found');
        return;
      }

      try {
        if (client.isRunning()) {
          await client.restart();
          // ToDo: refactor it on the server side.
          // Do not touch watchers on client side, just simplify the restart of the server.
          const configFiles = await findOxlintrcConfigFiles();
          await sendDidChangeWatchedFilesNotificationWith(client, configFiles);

          window.showInformationMessage('oxc server restarted.');
        } else {
          await startClient(client);
        }
      } catch (err) {
        client.error('Restarting client failed', err, 'force');
      }
    },
  );

export const toggleEnable = (clientGetter: LanguageClientGetter, config: Config) =>
  commands.registerCommand(
    OxcCommands.ToggleEnable,
    async () => {
      const client = clientGetter();
      await config.updateEnable(!config.enable);

      if (client.isRunning()) {
        if (!config.enable) {
          await client.stop();
        }
      } else {
        if (config.enable) {
          await startClient(client);
        }
      }
    },
  );

export const applyAllFixesFile = (clientGetter: LanguageClientGetter) =>
  commands.registerCommand(
    OxcCommands.ApplyAllFixesFile,
    async () => {
      const client = clientGetter();
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

export const showOutputCommand = (clientGetter: LanguageClientGetter) =>
  commands.registerCommand(
    OxcCommands.ShowOutputChannel,
    () => {
      const client = clientGetter();
      client.outputChannel.show();
    },
  );
