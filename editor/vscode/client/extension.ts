
import {
  ExtensionContext,
  window,
  commands,
  workspace,
} from "vscode";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

import { join } from 'node:path'

const languageClientId = 'oxc-client';
const languageClientName = 'oxc';
const outputChannelName = 'oxc';
const traceOutputChannelName = 'oxc.trace';

const enum OxcCommands {
  RestartServer    = "oxc.restartServer",
  ApplyAllFixes    = "oxc.applyAllFixes",
  ShowOutputChannel = "oxc.showOutputChannel",
  ShowTraceOutputChannel = "oxc.showTraceOutputChannel"
};


let client: LanguageClient;


export async function activate(context: ExtensionContext) {
  
  const restartCommand = commands.registerCommand(OxcCommands.RestartServer, async () => {
    if(!client) {
      window.showErrorMessage("oxc client not found");
      return
    }

    try {
			if (client.isRunning()) {
				await client.restart();

        window.showInformationMessage("oxc server restarted.");
			} else {
				await client.start();
			}
		} catch (err) {
			client.error("Restarting client failed", err, "force");
		}
  })

  const showOutputCommand = commands.registerCommand(OxcCommands.ShowOutputChannel, () => {
    client?.outputChannel?.show()
  })

  const showTraceOutputCommand = commands.registerCommand(OxcCommands.ShowTraceOutputChannel, () => {
    client?.traceOutputChannel?.show()
  })

  context.subscriptions.push(
    restartCommand,
    showOutputCommand,
    showTraceOutputCommand
  );

  const outputChannel = window.createOutputChannel(outputChannelName);
  const traceOutputChannel = window.createOutputChannel(traceOutputChannelName);

  const command = process.env.NODE_ENV === 'production' 
                ? join(context.extensionPath, './target/release/oxc_vscode') 
                : process.env.SERVER_PATH_DEV ;

  window.showInformationMessage(`oxc server path: ${command}`);

  const run: Executable = {
    command: command!,
    options: {
      env: {
         ...process.env,
        RUST_LOG: 'debug',
      },
    },
  };
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };
  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [
      "typescript",
      "javascript",
      "typescriptreact",
      "javascriptreact",
    ].map(lang => ({ language: lang, scheme: "file" })),

    synchronize: {
      // Notify the server about file changes to '.clientrc files contained in the workspace
      fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
    },
    outputChannel,
    traceOutputChannel,
  };

  // Create the language client and start the client.
  client = new LanguageClient(languageClientId, languageClientName, serverOptions, clientOptions);

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
