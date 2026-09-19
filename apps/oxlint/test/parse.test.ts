import { join as pathJoin } from "node:path";
import { Worker } from "node:worker_threads";
import { describe, expect, it, vi } from "vitest";

const path = pathJoin(import.meta.dirname, "dummy.js");

describe("parse", () => {
  it("keeps working after the module registry is reset", async () => {
    // Rust keeps its state across a module reset, JS does not.
    // `RuleTester` must still get a usable buffer from the second module instance.
    const first = await import("../src-js/package/parse.ts");
    first.parse(path, "let a = 1;");

    vi.resetModules();

    const second = await import("../src-js/package/parse.ts");
    expect(second).not.toBe(first);
    const bufferId = second.parse(path, "let b = 2;");

    const { buffers } = await import("../src-js/plugins/lint.ts");
    const buffer = buffers[bufferId];
    expect(buffer).toBeDefined();

    // The view registered by the second instance holds the second parse
    const sourceCode = await import("../src-js/plugins/source_code.ts");
    sourceCode.resetSourceAndAst();
    sourceCode.setupSourceForFile(buffer!, false);
    sourceCode.initSourceText();
    expect(sourceCode.sourceText).toBe("let b = 2;");
    sourceCode.resetSourceAndAst();
  });

  it("keeps a view alive after the thread that created it has exited", async () => {
    // `ArrayBuffer.prototype.transfer` turns the external view into a transferable buffer that still
    // aliases Rust's memory. The block must outlive the worker thread that allocated it.
    const worker = new Worker(pathJoin(import.meta.dirname, "parse_worker.mjs"));
    const exited = new Promise<void>((resolve) => worker.once("exit", () => resolve()));
    const laundered = await new Promise<ArrayBuffer>((resolve, reject) => {
      worker.once("message", resolve);
      worker.once("error", reject);
    });
    await exited;

    expect(new Uint8Array(laundered)[4096]).toBe(0x5a);
  });
});
