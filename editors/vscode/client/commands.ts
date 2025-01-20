import { CodeAction, Command, commands, Disposable, window, workspace } from 'vscode';

import { CodeActionRequest, CodeActionTriggerKind, LanguageClient, Position, Range } from 'vscode-languageclient/node';
import { Config } from './Config';

const commandPrefix = 'oxc';

export const enum OxcCommands {
  RestartServer = `${commandPrefix}.restartServer`,
  ApplyAllFixesFile = `${commandPrefix}.applyAllFixesFile`,
  ShowOutputChannel = `${commandPrefix}.showOutputChannel`,
  ToggleEnable = `${commandPrefix}.toggleEnable`,
}

export const restartServerCommand = (client: LanguageClient): Disposable => {
  return commands.registerCommand(
    OxcCommands.RestartServer,
    async () => {
      if (!client) {
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
};

export const showOutputChannelCommand = (client: LanguageClient): Disposable => {
  return commands.registerCommand(
    OxcCommands.ShowOutputChannel,
    () => {
      client.outputChannel.show();
    },
  );
};

export const toggleEnabledCommand = (config: Config): Disposable => {
  return commands.registerCommand(
    OxcCommands.ToggleEnable,
    () => {
      config.updateEnable(!config.enable);
    },
  );
};

export const applyAllFixesFileCommand = (client: LanguageClient): Disposable => {
  return commands.registerCommand(
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

      const lastLine = textEditor.document.lineAt(textEditor.document.lineCount - 1);
      const codeActionResult = await client.sendRequest(CodeActionRequest.type, {
        textDocument: {
          uri: textEditor.document.uri.toString(),
        },
        range: Range.create(Position.create(0, 0), lastLine.range.end),
        context: {
          diagnostics: [],
          only: [],
          triggerKind: CodeActionTriggerKind.Invoked,
        },
      });
      const commandsOrCodeActions = await client.protocol2CodeConverter.asCodeActionResult(codeActionResult || []);

      await Promise.all(
        commandsOrCodeActions
          .map(async (codeActionOrCommand) => {
            // Commands are always applied. Regardless of whether it's a Command or CodeAction#command.
            if (isCommand(codeActionOrCommand)) {
              await commands.executeCommand(codeActionOrCommand.command, codeActionOrCommand.arguments);
            } else {
              // Only preferred edits are applied
              // LSP states edits must be run first, then commands
              if (codeActionOrCommand.edit && codeActionOrCommand.isPreferred) {
                await workspace.applyEdit(codeActionOrCommand.edit);
              }
              if (codeActionOrCommand.command) {
                await commands.executeCommand(
                  codeActionOrCommand.command.command,
                  codeActionOrCommand.command.arguments,
                );
              }
            }
          }),
      );

      function isCommand(codeActionOrCommand: CodeAction | Command): codeActionOrCommand is Command {
        return typeof codeActionOrCommand.command === 'string';
      }
    },
  );
};
