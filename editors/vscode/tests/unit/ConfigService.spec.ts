import { strictEqual } from "assert";
import { workspace } from "vscode";
import { ConfigService } from "../../client/ConfigService.js";
import { WORKSPACE_FOLDER, WORKSPACE_SECOND_FOLDER } from "../test-helpers.js";

const conf = workspace.getConfiguration("oxc");

suite("ConfigService", () => {
  setup(async () => {
    const keys = ["path.server", "path.oxlint", "path.oxfmt", "path.tsgolint"];

    await Promise.all(keys.map((key) => conf.update(key, undefined)));
  });

  teardown(async () => {
    const keys = ["path.server", "path.oxlint", "path.oxfmt", "path.tsgolint"];

    await Promise.all(keys.map((key) => conf.update(key, undefined)));
  });

  const getWorkspaceFolderPlatformSafe = (folder = WORKSPACE_FOLDER) => {
    let workspace_path = folder.uri.path;
    if (process.platform === "win32") {
      workspace_path = workspace_path.replaceAll("/", "\\");
      if (workspace_path.startsWith("\\")) {
        workspace_path = workspace_path.slice(1);
      }
    }
    return workspace_path;
  };

  const createWorkspaceFolderFileUri = async (relativePath: string, folder = WORKSPACE_FOLDER) => {
    const workspace_path = getWorkspaceFolderPlatformSafe(folder);
    const path =
      process.platform === "win32"
        ? `${workspace_path}\\${relativePath}`
        : `${workspace_path}/${relativePath}`;

    await workspace.fs.writeFile(folder.uri.with({ path }), new Uint8Array());
  };

  const deleteWorkspaceFolderFileUri = async (relativePath: string, folder = WORKSPACE_FOLDER) => {
    const workspace_path = getWorkspaceFolderPlatformSafe(folder);
    const path =
      process.platform === "win32"
        ? `${workspace_path}\\${relativePath}`
        : `${workspace_path}/${relativePath}`;

    await workspace.fs.delete(folder.uri.with({ path }));
  };

  suite("getOxfmtServerBinPath", () => {
    test("resolves relative server path with workspace folder", async () => {
      const service = new ConfigService();
      const workspace_path = getWorkspaceFolderPlatformSafe();
      const nonDefinedServerPath = await service.getOxfmtServerBinPath();

      await createWorkspaceFolderFileUri("absolute/oxfmt");
      await createWorkspaceFolderFileUri("relative/oxfmt");

      strictEqual(nonDefinedServerPath, undefined);

      await conf.update("path.oxfmt", `${workspace_path}/absolute/oxfmt`);
      const absoluteServerPath = await service.getOxfmtServerBinPath();

      strictEqual(absoluteServerPath, `${workspace_path}/absolute/oxfmt`);

      await conf.update("path.oxfmt", "./relative/oxfmt");
      const relativeServerPath = await service.getOxfmtServerBinPath();

      strictEqual(relativeServerPath, `${workspace_path}/relative/oxfmt`);

      await deleteWorkspaceFolderFileUri("absolute/oxfmt");
      await deleteWorkspaceFolderFileUri("relative/oxfmt");
    });

    test("returns undefined for unsafe server path", async () => {
      await createWorkspaceFolderFileUri("../unsafe/oxfmt");
      const service = new ConfigService();
      await conf.update("path.oxfmt", "../unsafe/oxfmt");
      const unsafeServerPath = await service.getOxfmtServerBinPath();

      strictEqual(unsafeServerPath, undefined);
      await deleteWorkspaceFolderFileUri("../unsafe/oxfmt");
    });

    test("returns backslashes path on Windows", async () => {
      if (process.platform !== "win32") {
        return;
      }
      const service = new ConfigService();
      await conf.update("path.oxfmt", "./relative/oxfmt");
      const relativeServerPath = await service.getOxfmtServerBinPath();
      const workspace_path = getWorkspaceFolderPlatformSafe();

      strictEqual(
        workspace_path[1],
        ":",
        "The test workspace folder must be an absolute path with a drive letter on Windows",
      );
      strictEqual(relativeServerPath, `${workspace_path}\\relative\\oxfmt`);
    });
  });

  suite("getOxlintServerBinPath", () => {
    test("resolves relative server path with workspace folder", async () => {
      const service = new ConfigService();
      const nonDefinedServerPath = await service.getOxlintServerBinPath();
      const workspace_path = getWorkspaceFolderPlatformSafe();

      await createWorkspaceFolderFileUri("absolute/oxlint");
      await createWorkspaceFolderFileUri("relative/oxlint");

      strictEqual(nonDefinedServerPath, undefined);

      await conf.update("path.oxlint", `${workspace_path}/absolute/oxlint`);
      const absoluteServerPath = await service.getOxlintServerBinPath();

      strictEqual(absoluteServerPath, `${workspace_path}/absolute/oxlint`);

      await conf.update("path.oxlint", "./relative/oxlint");
      const relativeServerPath = await service.getOxlintServerBinPath();

      strictEqual(relativeServerPath, `${workspace_path}/relative/oxlint`);

      await deleteWorkspaceFolderFileUri("absolute/oxlint");
      await deleteWorkspaceFolderFileUri("relative/oxlint");
    });

    test("returns undefined for unsafe server path", async () => {
      await createWorkspaceFolderFileUri("../unsafe/oxlint");
      const service = new ConfigService();
      await conf.update("path.oxlint", "../unsafe/oxlint");
      const unsafeServerPath = await service.getOxlintServerBinPath();

      strictEqual(unsafeServerPath, undefined);
      await deleteWorkspaceFolderFileUri("../unsafe/oxlint");
    });

    test("returns backslashes path on Windows", async () => {
      if (process.platform !== "win32") {
        return;
      }
      const service = new ConfigService();
      await conf.update("path.oxlint", "./relative/oxlint");
      const relativeServerPath = await service.getOxlintServerBinPath();
      const workspace_path = getWorkspaceFolderPlatformSafe();

      strictEqual(
        workspace_path[1],
        ":",
        "The test workspace folder must be an absolute path with a drive letter on Windows",
      );
      strictEqual(relativeServerPath, `${workspace_path}\\relative\\oxlint`);
    });

    test("resolves binary path in multi-folder workspace", async () => {
      const service = new ConfigService();
      const workspace_path = getWorkspaceFolderPlatformSafe();

      await createWorkspaceFolderFileUri("node_modules/.bin/oxlint");
      await createWorkspaceFolderFileUri("node_modules/.bin/oxlint", WORKSPACE_SECOND_FOLDER);
      const absoluteServerPath = await service.getOxlintServerBinPath();

      strictEqual(absoluteServerPath, `${workspace_path}/node_modules/.bin/oxlint`);

      await deleteWorkspaceFolderFileUri("node_modules/.bin/oxlint");
      await deleteWorkspaceFolderFileUri("node_modules/.bin/oxlint", WORKSPACE_SECOND_FOLDER);
    });
  });
});
