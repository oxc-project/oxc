/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

import { act, cleanup, render } from "@testing-library/react";
import { install, type Clock } from "@sinonjs/fake-timers";
import { createRequire } from "node:module";
import { JSDOM } from "jsdom";
import React from "react";
import util from "util";
import { z } from "zod/v4";
import { fromZodError } from "zod-validation-error/v4";
import { initFbt, toJSON } from "./shared-runtime.ts";
import * as sharedRuntime from "./shared-runtime.ts";
import * as featureFlag from "./ReactForgetFeatureFlag.ts";
import useEffectWrapper from "./useEffectWrapper.ts";
import * as compilerRuntime from "./compiler-runtime.ts";

const nodeRequire = createRequire(import.meta.url);
// Keep the fixture module names used by upstream, without installing fake packages.
const require = (name: string): unknown => {
  switch (name) {
    case "shared-runtime":
      return sharedRuntime;
    case "ReactForgetFeatureFlag":
      return featureFlag;
    case "useEffectWrapper":
      return { __esModule: true, default: useEffectWrapper };
    case "react-compiler-runtime":
      return compilerRuntime;
    default:
      return nodeRequire(name);
  }
};

/**
 * Set up the global environment for JSDOM tests.
 * This is a hack to let us share code and setup between the test
 * and runner environments. As an alternative, we could evaluate all setup
 * in the jsdom test environment (which provides more isolation), but that
 * may be slower.
 */
const { window: testWindow } = new JSDOM(undefined);
(globalThis as any).document = testWindow.document;
(globalThis as any).window = testWindow.window;
(globalThis as any).React = React;
(globalThis as any).render = render;
initFbt();

(globalThis as any).placeholderFn = function (..._args: Array<any>) {
  throw new Error("Fixture not implemented!");
};
export type EvaluatorResult = {
  kind: "ok" | "exception" | "UnexpectedError";
  value: string;
  logs: Array<string>;
};

/**
 * Define types and schemas for fixture entrypoint
 */
const EntrypointSchema = z.strictObject({
  // Preserve exotic component objects, including their React tag and identity.
  // z.object({}) strips the properties React needs to render memo/forwardRef.
  fn: z.custom<React.ComponentType<any> | React.ExoticComponent<any>>(
    (value) =>
      typeof value === "function"
      || (typeof value === "object"
        && value !== null
        && "$$typeof" in value
        && [
          Symbol.for("react.memo"),
          Symbol.for("react.forward_ref"),
          Symbol.for("react.lazy"),
        ].includes(value.$$typeof as symbol)),
  ),
  params: z.array(z.any()),

  // DEPRECATED, unused
  isComponent: z.optional(z.boolean()),

  // if enabled, the `fn` is assumed to be a component and this is assumed
  // to be an array of props. the component is mounted once and rendered
  // once per set of props in this array.
  sequentialRenders: z.optional(z.nullable(z.array(z.any()))).default(null),
});
const ExportSchema = z.object({
  FIXTURE_ENTRYPOINT: EntrypointSchema,
});

const NO_ERROR_SENTINEL = Symbol();
class EvaluatorError extends Error {}

function exceptionValue(error: unknown): string {
  // Stack traces vary after compilation, but error types and non-Error thrown
  // values are observable and must not collapse to an undefined message.
  return error instanceof Error ? error.toString() : (toJSON(error) ?? String(error));
}
/**
 * Wraps WrapperTestComponent in an error boundary to simplify re-rendering
 * when an exception is thrown.
 * A simpler alternative may be to re-mount test components manually.
 */
class WrapperTestComponentWithErrorBoundary extends React.Component<
  { fn: any; params: Array<any> },
  { errorFromLastRender: any }
> {
  /**
   * Limit retries of the child component by caching seen errors.
   */
  propsErrorMap: Map<any, any>;
  lastProps: any;
  // lastProps: object | null;
  constructor(props: any) {
    super(props);
    this.lastProps = null;
    this.propsErrorMap = new Map<any, any>();
    this.state = {
      errorFromLastRender: NO_ERROR_SENTINEL,
    };
  }
  static getDerivedStateFromError(error: any) {
    // Reschedule a second render that immediately returns the cached error
    return { errorFromLastRender: error };
  }
  override componentDidUpdate() {
    if (this.state.errorFromLastRender !== NO_ERROR_SENTINEL) {
      // Reschedule a third render that immediately returns the cached error
      // oxlint-disable-next-line react/no-did-update-set-state -- Match the upstream error-boundary retry protocol.
      this.setState({ errorFromLastRender: NO_ERROR_SENTINEL });
    }
  }
  override render() {
    if (this.state.errorFromLastRender !== NO_ERROR_SENTINEL && this.props === this.lastProps) {
      /**
       * The last render errored, cache the error message to avoid running the
       * test fixture more than once
       */
      const errorMsg = `[[ (exception in render) ${exceptionValue(this.state.errorFromLastRender)} ]]`;
      this.propsErrorMap.set(this.lastProps, errorMsg);
      return errorMsg;
    }
    this.lastProps = this.props;
    const cachedError = this.propsErrorMap.get(this.props);
    if (cachedError != null) {
      return cachedError;
    }
    return React.createElement(WrapperTestComponent, this.props);
  }
}

function WrapperTestComponent(props: { fn: any; params: Array<any> }) {
  if (typeof props.fn === "object" || props.fn.prototype?.isReactComponent) {
    return React.createElement(props.fn, props.params[0]);
  }
  const result = props.fn(...props.params);
  // Hacky solution to determine whether the fixture returned jsx (which
  // needs to passed through to React's runtime as-is) or a non-jsx value
  // (which should be converted to a string).
  const containsElement = (value: unknown): boolean =>
    React.isValidElement(value) || (Array.isArray(value) && value.some(containsElement));
  if (containsElement(result)) {
    return result;
  } else {
    return toJSON(result);
  }
}

async function settle(action: () => void, clock: Clock): Promise<void> {
  await act(async () => {
    action();
    // Keep React's act environment and fixture capture active for promise work
    // and Node's unhandled-rejection notification. act then flushes the updates
    // triggered by those continuations before resolving.
    await new Promise<void>((done) => setImmediate(done));
  });
  // Timers run on a fixture-local clock, never on the workers' wall clocks.
  // React effects can schedule more timers while applying a timer's updates.
  for (let batch = 0; clock.countTimers() > 0; batch++) {
    if (batch === 100) throw new EvaluatorError("Fixture timers did not settle");
    try {
      // oxlint-disable-next-line no-await-in-loop -- Commit each timer batch before observing newly scheduled effects.
      await act(async () => {
        await clock.runAllAsync();
      });
    } catch (error) {
      throw new EvaluatorError("Fixture timers failed: " + exceptionValue(error));
    }
  }
}

async function renderComponentSequentiallyForEachProps(
  fn: any,
  sequentialRenders: Array<any>,
  clock: Clock,
): Promise<string> {
  if (sequentialRenders.length === 0) {
    throw new Error("Expected at least one set of props when using `sequentialRenders`");
  }
  const initialProps = sequentialRenders[0]!;
  const results = [];
  let rendered!: ReturnType<typeof render>;
  await settle(() => {
    rendered = render(
      React.createElement(WrapperTestComponentWithErrorBoundary, {
        fn,
        params: [initialProps],
      }),
    );
  }, clock);
  results.push(rendered.container.innerHTML);

  for (let i = 1; i < sequentialRenders.length; i++) {
    // oxlint-disable-next-line no-await-in-loop -- Observe each committed render before changing props.
    await settle(() => {
      rendered.rerender(
        React.createElement(WrapperTestComponentWithErrorBoundary, {
          fn,
          params: [sequentialRenders[i]],
        }),
      );
    }, clock);
    results.push(rendered.container.innerHTML);
  }
  return results.join("\n");
}

type FixtureEvaluatorResult = Omit<EvaluatorResult, "logs">;
(globalThis as any).evaluateFixtureExport = async function (
  exports: unknown,
  clock: Clock,
): Promise<FixtureEvaluatorResult> {
  const parsedExportResult = ExportSchema.safeParse(exports);
  if (!parsedExportResult.success) {
    const exportDetail =
      typeof exports === "object" && exports != null
        ? `object ${util.inspect(exports)}`
        : String(exports);
    return {
      kind: "UnexpectedError",
      value: `${fromZodError(parsedExportResult.error)}\nFound ` + exportDetail,
    };
  }
  const entrypoint = parsedExportResult.data.FIXTURE_ENTRYPOINT;
  if (entrypoint.sequentialRenders !== null) {
    const result = await renderComponentSequentiallyForEachProps(
      entrypoint.fn,
      entrypoint.sequentialRenders,
      clock,
    );

    return {
      kind: "ok",
      value: result ?? "null",
    };
  } else {
    let rendered!: ReturnType<typeof render>;
    await settle(() => {
      rendered = render(React.createElement(WrapperTestComponent, entrypoint));
    }, clock);

    return {
      kind: "ok",
      value: rendered.container.innerHTML,
    };
  }
};

export async function doEval(source: string): Promise<EvaluatorResult> {
  "use strict";

  const clock = install({
    now: 0,
    toFake: ["setTimeout", "clearTimeout", "setInterval", "clearInterval", "Date", "performance"],
    loopLimit: 100,
  });
  const originalConsole = globalThis.console;
  const asynchronousErrors: unknown[] = [];
  const onUnhandledRejection = (error: unknown) => asynchronousErrors.push(error);
  process.on("unhandledRejection", onUnhandledRejection);
  const logs: Array<string> = [];
  const mockedLog = (...args: Array<any>) => {
    logs.push(
      args
        .map((arg) => {
          if (arg instanceof Error) {
            return arg.toString();
          } else {
            return util.inspect(arg);
          }
        })
        .join(","),
    );
  };

  (globalThis.console as any) = {
    ...originalConsole,
    info: mockedLog,
    log: mockedLog,
    warn: mockedLog,
    error: (...args: Array<any>) => {
      if (typeof args[0] === "string" && args[0].includes("ReactDOMTestUtils.act` is deprecated")) {
        // remove this once @testing-library/react is upgraded to React 19.
        return;
      }

      const stack = new Error().stack?.split("\n", 5) ?? [];
      for (const stackFrame of stack) {
        // React warns on exceptions thrown during render, we avoid printing
        // here to reduce noise in test fixture outputs.
        if (
          (stackFrame.includes("at logCaughtError")
            && stackFrame.includes("react-dom-client.development.js"))
          || (stackFrame.includes("at defaultOnRecoverableError")
            && stackFrame.includes("react-dom-client.development.js"))
        ) {
          return;
        }
      }
      mockedLog(...args);
    },
    table: mockedLog,
    trace: () => {},
  };
  try {
    let result: FixtureEvaluatorResult;
    try {
      // Evaluate CommonJS with an explicit resolver for fixture runtime modules.
      // oxlint-disable-next-line typescript/no-implied-eval -- Executing fixture programs is the purpose of this harness.
      result = await new Function(
        "require",
        "exceptionValue",
        "EvaluatorError",
        "clock",
        `
    "use strict";
    return (async () => {
      // Exports should be overwritten by source
      let exports = {
        FIXTURE_ENTRYPOINT: {
          fn: globalThis.placeholderFn,
          params: [],
        },
      };
      const module = {exports};
      let reachedInvoke = false;
      try {
        // run in an iife to avoid naming collisions
        (() => {${source}})();
        reachedInvoke = true;
        if (exports.FIXTURE_ENTRYPOINT?.fn === globalThis.placeholderFn) {
          return {
            kind: "UnexpectedError",
            value: "Fixture not implemented",
          };
        }
        return await evaluateFixtureExport(exports, clock);
      } catch (e) {
        if (!reachedInvoke || e instanceof EvaluatorError) {
          return {
            kind: "UnexpectedError",
            value: exceptionValue(e),
          };
        } else {
          return {
            kind: "exception",
            value: exceptionValue(e),
          };
        }
      }
    })()`,
      )(require, exceptionValue, EvaluatorError, clock);
    } catch (e) {
      // Syntax and module-initialization failures belong to the harness rather
      // than the observable exceptions thrown by a fixture's entrypoint.
      result = {
        kind: "UnexpectedError",
        value:
          "Unexpected error during eval, possible syntax error?\n"
          + (e instanceof Error ? e.message : String(e))
          + "\n\nsource:\n"
          + source,
      };
    }

    try {
      await settle(cleanup, clock);
    } catch (e) {
      result = { kind: "UnexpectedError", value: "Fixture cleanup failed: " + String(e) };
    }
    if (asynchronousErrors.length > 0) {
      result = {
        kind: "UnexpectedError",
        value: "Unhandled fixture rejection: " + asynchronousErrors.map(String).join("\n"),
      };
    }
    return { ...result, logs };
  } finally {
    try {
      clock.uninstall();
    } finally {
      process.removeListener("unhandledRejection", onUnhandledRejection);
      globalThis.console = originalConsole;
    }
  }
}
