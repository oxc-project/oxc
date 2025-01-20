import { promises as fsPromises } from 'node:fs';
import { join } from 'node:path';

import { ExtensionContext, workspace } from 'vscode';

import { Config } from './Config';

export default async function findBinary(context: ExtensionContext, config: Config): Promise<string> {
  let bin = config.binPath;
  if (bin) {
    try {
      await fsPromises.access(bin);
      return bin;
    } catch {}
  }

  const workspaceFolders = workspace.workspaceFolders;
  const isWindows = process.platform === 'win32';

  if (workspaceFolders?.length && !isWindows) {
    try {
      return await Promise.any(
        workspaceFolders.map(async (folder) => {
          const binPath = join(
            folder.uri.fsPath,
            'node_modules',
            '.bin',
            'oxc_language_server',
          );

          await fsPromises.access(binPath);
          return binPath;
        }),
      );
    } catch {}
  }

  const ext = isWindows ? '.exe' : '';
  // NOTE: The `./target/release` path is aligned with the path defined in .github/workflows/release_vscode.yml
  return (
    process.env.SERVER_PATH_DEV ??
      join(context.extensionPath, `./target/release/oxc_language_server${ext}`)
  );
}
