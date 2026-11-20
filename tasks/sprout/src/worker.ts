import { parentPort } from "node:worker_threads";
import { doEval } from "./evaluator.ts";
import { evaluatorCode } from "./transform.ts";

export type Fixture = { path: string; source: string; code: string | null; diagnostics: string[] };

parentPort!.on(
  "message",
  async ({ fixture, compiled }: { fixture: Fixture; compiled: boolean }) => {
    try {
      const source = compiled ? fixture.code : fixture.source;
      if (source === null) throw new Error(fixture.diagnostics.join("\n"));
      (globalThis as any).__SNAP_EVALUATOR_MODE = compiled ? "forget" : undefined;
      parentPort!.postMessage(await doEval(evaluatorCode(source, fixture.path)));
    } catch (error) {
      parentPort!.postMessage({ kind: "UnexpectedError", value: String(error), logs: [] });
    } finally {
      (globalThis as any).__SNAP_EVALUATOR_MODE = undefined;
    }
  },
);
