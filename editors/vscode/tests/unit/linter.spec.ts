import { ok } from "assert";
import { ExtensionContext, FileSystemWatcher, workspace } from "vscode";
import LinterTool from "../../client/tools/linter.js";

suite("LinterTool", () => {
  test("watches oxlint.config.ts when requireConfig is enabled", () => {
    const tool = new LinterTool();
    const createdPatterns: string[] = [];
    const originalWatcher = workspace.createFileSystemWatcher;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any -- test stub
    (workspace as any).createFileSystemWatcher = (pattern: string) => {
      createdPatterns.push(pattern.toString());
      return {
        onDidCreate: (_listener?: unknown) => ({ dispose() {} }),
        onDidDelete: (_listener?: unknown) => ({ dispose() {} }),
        dispose() {},
      } as FileSystemWatcher;
    };

    try {
      const config = { requireConfig: true, enable: true } as unknown as {
        requireConfig: boolean;
        enable: boolean;
      };
      const context = { subscriptions: [] } as ExtensionContext;
      const statusBarItemHandler = { updateTool: () => {} } as unknown as {
        updateTool: () => void;
      };

      tool.generateActivatorByConfig(config, context, statusBarItemHandler);
    } finally {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any -- test cleanup
      (workspace as any).createFileSystemWatcher = originalWatcher;
    }

    ok(
      createdPatterns.some((pattern) => pattern.includes(".oxlintrc.json")),
      "expected watcher for .oxlintrc.json",
    );
    ok(
      createdPatterns.some((pattern) => pattern.includes("oxlint.config.ts")),
      "expected watcher for oxlint.config.ts",
    );
  });
});
