import { parentPort, workerData } from "node:worker_threads";
import { registerJsPluginWorker } from "./bindings.js";
import { lintFile, loadPlugin, setupRuleConfigs } from "./plugins/index.ts";
import { getErrorMessage } from "./utils/utils.ts";

interface PluginLoadRecord {
  pluginUrl: string;
  pluginName: string | null;
  pluginNameIsAlias: boolean;
  workspaceUri: string | null;
  expected: {
    name: string;
    offset: number;
    ruleNames: string[];
  };
}

interface WorkerBootstrap {
  plugins: PluginLoadRecord[];
  optionsJson: string;
}

type LoadPluginReturnValue = { Success: PluginLoadRecord["expected"] } | { Failure: string };

interface PluginWorkerData {
  poolId: number;
  workerIndex: number;
  bootstrapJSON: string;
}

const { poolId, workerIndex, bootstrapJSON } = workerData as PluginWorkerData;

async function initialize(): Promise<void> {
  const bootstrap = JSON.parse(bootstrapJSON) as WorkerBootstrap;

  // Plugin imports are deliberately sequential. Rule IDs and module side effects must occur in
  // exactly the same order as on the canonical main isolate.
  for (const plugin of bootstrap.plugins) {
    // oxlint-disable-next-line no-await-in-loop -- Preserve the canonical plugin registration order.
    const resultJSON = await loadPlugin(
      plugin.pluginUrl,
      plugin.pluginName,
      plugin.pluginNameIsAlias,
      plugin.workspaceUri,
    );
    const result = JSON.parse(resultJSON) as LoadPluginReturnValue;
    if ("Failure" in result) throw new Error(result.Failure);

    if (JSON.stringify(result.Success) !== JSON.stringify(plugin.expected)) {
      throw new Error(
        `Plugin registration did not match the main isolate. Expected ${JSON.stringify(plugin.expected)}, received ${JSON.stringify(result.Success)}`,
      );
    }
  }

  const setupError = setupRuleConfigs(bootstrap.optionsJson);
  if (setupError !== null) throw new Error(setupError);

  registerJsPluginWorker(poolId, workerIndex, lintFileWrapper);
}

function lintFileWrapper(
  filePath: string,
  bufferId: number,
  buffer: Uint8Array | null | undefined,
  ruleIds: number[],
  optionsIds: number[],
  settingsJSON: string,
  globalsJSON: string,
  workspaceUri: string | null | undefined,
): string | null {
  if (buffer === undefined) throw new TypeError("`buffer` should not be `undefined`");
  if (workspaceUri === undefined) {
    throw new TypeError("`workspaceUri` should not be `undefined`");
  }
  return lintFile(
    filePath,
    bufferId,
    buffer,
    ruleIds,
    optionsIds,
    settingsJSON,
    globalsJSON,
    workspaceUri,
  );
}

try {
  await initialize();
  parentPort!.postMessage({ ready: true });
} catch (error) {
  parentPort!.postMessage({ error: getErrorMessage(error) });
}
