import { afterEach, describe, expect, it } from "vitest";
import vm from "node:vm";

const runtimeFlagsModulePath = "../src-js/runtime_flags.ts";

const hadDebug = Reflect.has(globalThis, "DEBUG");
const hadConformance = Reflect.has(globalThis, "CONFORMANCE");
const previousDebug = (globalThis as typeof globalThis & { DEBUG?: boolean }).DEBUG;
const previousConformance = (globalThis as typeof globalThis & { CONFORMANCE?: boolean }).CONFORMANCE;

afterEach(() => {
  const globalObj = globalThis as typeof globalThis & { DEBUG?: boolean; CONFORMANCE?: boolean };

  if (hadDebug) {
    globalObj.DEBUG = previousDebug;
  } else {
    delete globalObj.DEBUG;
  }

  if (hadConformance) {
    globalObj.CONFORMANCE = previousConformance;
  } else {
    delete globalObj.CONFORMANCE;
  }
});

describe("runtime flags", () => {
  it("parses raw-source flag env values", async () => {
    const { isRuntimeFlagEnabled } = await import(runtimeFlagsModulePath);

    expect(isRuntimeFlagEnabled(undefined)).toBe(false);
    expect(isRuntimeFlagEnabled("0")).toBe(false);
    expect(isRuntimeFlagEnabled("false")).toBe(false);
    expect(isRuntimeFlagEnabled("1")).toBe(true);
    expect(isRuntimeFlagEnabled("true")).toBe(true);
  });

  it("fills missing flags without overwriting explicit globals", async () => {
    const { installRuntimeFlags } = await import(runtimeFlagsModulePath);
    const globalObj: { DEBUG?: boolean; CONFORMANCE?: boolean } = { DEBUG: true };

    installRuntimeFlags(
      {
        DEBUG: "false",
        CONFORMANCE: "1",
      } as NodeJS.ProcessEnv,
      globalObj as typeof globalThis & { DEBUG?: boolean; CONFORMANCE?: boolean },
    );

    expect(globalObj).toEqual({
      DEBUG: true,
      CONFORMANCE: true,
    });
  });

  it("creates bare runtime flag bindings visible to later raw-source modules", async () => {
    const { installRuntimeFlags } = await import(runtimeFlagsModulePath);
    const context = vm.createContext({}) as typeof globalThis & {
      DEBUG?: boolean;
      CONFORMANCE?: boolean;
      eval?: (code: string) => unknown;
      globalThis: object;
    };

    context.globalThis = context;
    context.eval = (code) => vm.runInContext(code, context);

    installRuntimeFlags({ DEBUG: "1", CONFORMANCE: "0" } as NodeJS.ProcessEnv, context);

    const flags = vm.runInContext("({ debug: DEBUG, conformance: CONFORMANCE })", context) as {
      debug: boolean;
      conformance: boolean;
    };

    expect(flags).toEqual({
      debug: true,
      conformance: false,
    });
  });
});
