import assert from "node:assert/strict";
import { test } from "node:test";
import { Worker } from "node:worker_threads";

import { doEval, type EvaluatorResult } from "../src/evaluator.ts";
import { runSprout } from "../src/index.ts";
import { evaluatorCode } from "../src/transform.ts";

const evaluate = (source: string) => doEval(evaluatorCode(source, "evaluator.tsx"));

function asynchronousComponent(value: string, sequential = false): string {
  return `
    import {useEffect, useState} from 'react';
    function Component({value}) {
      const [state, setState] = useState(0);
      useEffect(() => {
        Promise.resolve().then(() => Promise.resolve()).then(() => setState(${value}));
      }, [value]);
      return <span>{state}</span>;
    }
    export const FIXTURE_ENTRYPOINT = {
      fn: Component,
      params: [{value: 1}],
      ${sequential ? "sequentialRenders: [{value:1}, {value:1}, {value:2}, {value:1}]," : ""}
    };
  `;
}

void test("single renders capture promise-driven React state updates", async () => {
  const result = await evaluate(asynchronousComponent("value"));
  assert.equal(result.kind, "ok");
  assert.equal(result.value, "<span>1</span>");
  assert.deepEqual(result.logs, []);
  const comparison = await runSprout(
    evaluatorCode(asynchronousComponent("value"), "original.tsx"),
    evaluatorCode(asynchronousComponent("value + 1"), "compiled.tsx"),
  );
  assert.equal(comparison.kind, "invalid");
  assert.match(comparison.value, /<span>1<\/span>/);
  assert.match(comparison.value, /<span>2<\/span>/);
});

void test("each sequential render settles before its HTML is captured", async () => {
  const result = await evaluate(asynchronousComponent("value", true));
  assert.equal(result.kind, "ok");
  assert.equal(result.value, "<span>1</span>\n<span>1</span>\n<span>2</span>\n<span>1</span>");
  assert.deepEqual(result.logs, []);
});

void test("timer-driven updates use the same virtual clock in both lanes", async () => {
  const original = asynchronousComponent("value", true).replace(
    "Promise.resolve().then(() => Promise.resolve()).then(() => setState(value))",
    "setTimeout(() => setState(value), 50)",
  );
  const result = await evaluate(original);
  assert.equal(result.kind, "ok");
  assert.equal(result.value, "<span>1</span>\n<span>1</span>\n<span>2</span>\n<span>1</span>");
  const comparison = await runSprout(
    evaluatorCode(original, "original.tsx"),
    evaluatorCode(original.replace("setState(value)", "setState(value + 1)"), "compiled.tsx"),
  );
  assert.equal(comparison.kind, "invalid");
  assert.match(comparison.value, /<span>3<\/span>/);
});

void test("clock reads are deterministic and each evaluation starts at zero", async () => {
  const source = `
    function Component() { return [Date.now(), performance.now()]; }
    export const FIXTURE_ENTRYPOINT = {fn:Component, params:[]};
  `;
  assert.equal((await evaluate(source)).value, "[0,0]");
  assert.equal((await evaluate(source)).value, "[0,0]");
});

void test("invalid object entrypoints fail instead of matching React render errors", async () => {
  const result = await evaluate("export const FIXTURE_ENTRYPOINT = {fn: {}, params: []};");
  assert.equal(result.kind, "UnexpectedError");
});

for (const sequential of [false, true]) {
  void test(`thrown values and error types are compared on the ${sequential ? "sequential" : "single"} path`, async () => {
    function throwing(value: string): string {
      return `
        function Component() { throw ${value}; }
        export const FIXTURE_ENTRYPOINT = {
          fn: Component, params: [], ${sequential ? "sequentialRenders: [{}]," : ""}
        };
      `;
    }
    for (const [original, compiled] of [
      ['"first"', '"second"'],
      ["{code:1}", "{code:2}"],
      ['new Error("same")', 'new TypeError("same")'],
    ]) {
      // oxlint-disable-next-line no-await-in-loop -- Evaluations share React and console state.
      const result = await runSprout(
        evaluatorCode(throwing(original!), "original.tsx"),
        evaluatorCode(throwing(compiled!), "compiled.tsx"),
      );
      assert.equal(result.kind, "invalid", `${original} versus ${compiled}: ${result.value}`);
    }
    const same = evaluatorCode(throwing("{code:1}"), "same.tsx");
    assert.equal((await runSprout(same, same)).kind, "success");
  });
}

function asynchronousCleanup(value: string, sequential = false): string {
  return `
    import {useEffect} from 'react';
    function Component({value}) {
      useEffect(() => () => {
        Promise.resolve().then(() => Promise.resolve()).then(() => console.log(${value}));
      }, [value]);
      return <span>{value}</span>;
    }
    export const FIXTURE_ENTRYPOINT = {
      fn: Component, params: [{value:1}],
      ${sequential ? "sequentialRenders: [{value:1}, {value:2}]," : ""}
    };
  `;
}

void test("promise continuations from final cleanup stay in the fixture's capture", async () => {
  const result = await evaluate(asynchronousCleanup("value"));
  assert.equal(result.kind, "ok");
  assert.deepEqual(result.logs, ["1"]);
  const comparison = await runSprout(
    evaluatorCode(asynchronousCleanup("value"), "original.tsx"),
    evaluatorCode(asynchronousCleanup("value + 1"), "compiled.tsx"),
  );
  assert.equal(comparison.kind, "invalid");
  assert.match(comparison.value, /logs: \[1\]/);
  assert.match(comparison.value, /logs: \[2\]/);
});

void test("sequential effect cleanups settle before final cleanup", async () => {
  const result = await evaluate(asynchronousCleanup("value", true));
  assert.equal(result.kind, "ok");
  assert.deepEqual(result.logs, ["1", "2"]);
});

void test("timers scheduled by final cleanup remain inside console capture", async () => {
  const original = asynchronousCleanup("value").replace(
    "Promise.resolve().then(() => Promise.resolve()).then(() => console.log(value))",
    "setTimeout(() => console.log(value), 10)",
  );
  const result = await evaluate(original);
  assert.equal(result.kind, "ok");
  assert.deepEqual(result.logs, ["1"]);
  const comparison = await runSprout(
    evaluatorCode(original, "original.tsx"),
    evaluatorCode(original.replace("console.log(value)", "console.log(value + 1)"), "compiled.tsx"),
  );
  assert.equal(comparison.kind, "invalid");
});

void test("non-terminating timer work fails instead of comparing identical harness errors", async () => {
  const source = `
    import {useEffect} from 'react';
    function Component() {
      useEffect(() => {
        const timer = setInterval(() => {}, 1);
        return () => clearInterval(timer);
      }, []);
      return null;
    }
    export const FIXTURE_ENTRYPOINT = {fn:Component, params:[]};
  `;
  const result = await evaluate(source);
  assert.equal(result.kind, "UnexpectedError");
  assert.match(result.value, /Fixture timers failed/);
  const healthy = await evaluate("export const FIXTURE_ENTRYPOINT = {fn:()=>42, params:[]};");
  assert.equal(healthy.kind, "ok");
  assert.equal(healthy.value, "42");
});

for (const kind of ["memo", "forwardRef", "lazy", "class"] as const) {
  function objectComponent(value: string, sequential: boolean): string {
    const body = `function Component({value}${kind === "forwardRef" ? ", ref" : ""}) {
      return <span>{${value}}</span>;
    }`;
    return `
      const Component = ${
        kind === "class"
          ? `class extends React.Component { render() { const {value} = this.props; return <span>{${value}}</span>; } }`
          : kind === "lazy"
            ? `React.lazy(() => Promise.resolve({default: ${body}}))`
            : `React.${kind}(${body})`
      };
      const first = {value:1};
      export const FIXTURE_ENTRYPOINT = {
        fn: Component, params: [first],
        ${sequential ? "sequentialRenders: [first, first, {value:2}, first]," : ""}
      };
    `;
  }
  for (const sequential of [false, true]) {
    void test(`${kind} objects render on the ${sequential ? "sequential" : "single"} path`, async () => {
      const result = await evaluate(objectComponent("value", sequential));
      assert.equal(result.kind, "ok");
      assert.equal(
        result.value,
        sequential
          ? "<span>1</span>\n<span>1</span>\n<span>2</span>\n<span>1</span>"
          : "<span>1</span>",
      );
      assert.deepEqual(result.logs, []);
      const comparison = await runSprout(
        evaluatorCode(objectComponent("value", sequential), "original.tsx"),
        evaluatorCode(objectComponent("value + 1", sequential), "compiled.tsx"),
      );
      assert.equal(comparison.kind, "invalid");
      assert.match(comparison.value, sequential ? /<span>3<\/span>/ : /<span>2<\/span>/);
    });
  }
}

void test("cleanup rejections fail their own evaluation and cannot contaminate the next one", async () => {
  const worker = new Worker(new URL("../src/worker.ts", import.meta.url));
  try {
    const evaluateInWorker = (source: string) =>
      new Promise<EvaluatorResult>((resolve, reject) => {
        worker.once("error", reject);
        worker.once("message", (result) => {
          worker.removeListener("error", reject);
          resolve(result);
        });
        worker.postMessage({
          fixture: { path: "cleanup.tsx", source, code: source, diagnostics: [] },
          compiled: false,
        });
      });
    const result = await evaluateInWorker(`
      import {useEffect} from 'react';
      function Component() {
        useEffect(() => () => {
          Promise.resolve().then(() => { throw new Error('cleanup rejection'); });
        }, []);
        return null;
      }
      export const FIXTURE_ENTRYPOINT = {fn:Component, params:[]};
    `);
    assert.equal(result.kind, "UnexpectedError");
    assert.match(result.value, /cleanup rejection/);
    const next = await evaluateInWorker(`
      function Component() { return 'next'; }
      export const FIXTURE_ENTRYPOINT = {fn:Component, params:[]};
    `);
    assert.equal(next.kind, "ok");
    assert.deepEqual(next.logs, []);
  } finally {
    await worker.terminate();
  }
});
