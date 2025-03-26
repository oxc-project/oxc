import { FileChangeType, Uri, workspace } from 'vscode';
import { DidChangeWatchedFilesNotification, LanguageClient } from 'vscode-languageclient/node';
import { oxlintConfigFileName } from './Config';

export async function findOxlintrcConfigFiles() {
  return workspace.findFiles(`**/${oxlintConfigFileName}`);
}

// Starts the client, it does not check if it is already started
export async function startClient(client: LanguageClient) {
  await client.start();
  // ToDo: refactor it on the server side.
  // Do not touch watchers on client side, just simplify the start of the server.
  const configFiles = await findOxlintrcConfigFiles();
  await sendDidChangeWatchedFilesNotificationWith(client, configFiles);
}

export async function sendDidChangeWatchedFilesNotificationWith(languageClient: LanguageClient, files: Uri[]) {
  await languageClient.sendNotification(DidChangeWatchedFilesNotification.type, {
    changes: files.map(file => {
      return { uri: file.toString(), type: FileChangeType.Created };
    }),
  });
}
