import { availableParallelism } from "node:os";
import { createBuffer } from "../../parser/raw-transfer/common.js";
import {
  lint,
  type ExternalLinterCb,
  type ExternalLinterLoadPluginCb,
  type PluginLoadResult,
} from "./binding.js";

class PluginRegistry extends Map {}

const pluginRegistry = new PluginRegistry();

class AllocatorPool {
  private allocators: Uint8Array[];

  constructor(size: number) {
    this.allocators = Array.from({ length: size }, () => createBuffer());
  }

  getAllocator(index: number): Uint8Array {
    const allocator = this.allocators[index];
    if (!allocator) {
      throw new Error(`Allocator at index ${index} does not exist.`);
    }
    return allocator;
  }

  getAllocators(): Uint8Array[] {
    return this.allocators;
  }
}

class Linter {
  pluginRegistry: PluginRegistry = new PluginRegistry();
  allocatorPool: AllocatorPool;

  constructor() {
    this.allocatorPool = new AllocatorPool(availableParallelism());
  }

  run(): boolean {
    return lint(
      this.allocatorPool.getAllocators(),
      this.lint.bind(this),
      this.loadPlugin.bind(this)
    );
  }

  private loadPlugin: ExternalLinterLoadPluginCb = async (
    pluginName: string
  ): Promise<PluginLoadResult> => {
    try {
      const plugin = await import(pluginName);
      pluginRegistry.set(pluginName, plugin);
      return { type: "Success" };
    } catch (error) {
      return { type: "Failure", field0: (error as Error).message };
    }
  };

  private lint: ExternalLinterCb = (path: string, allocatorIdx: number) => {
    // TODO: do the things

    throw new Error("Linting is not implemented yet.");
  };
}

function main() {
  const linter = new Linter();

  const result = linter.run();

  if (!result) {
    process.exit(1);
  }
}

main();
