/**
 * Seed build-time flag globals when running source files directly.
 *
 * Bundled builds replace bare `DEBUG` / `CONFORMANCE` references at build time.
 * Raw-source entrypoints (for example `tsx --conditions=code ./src-js/cli.ts`)
 * still need those globals to exist before any flag-guarded modules execute.
 */

type RuntimeFlagsGlobal = typeof globalThis & {
  DEBUG?: boolean;
  CONFORMANCE?: boolean;
  eval?: (code: string) => unknown;
};

type RuntimeFlagName = "DEBUG" | "CONFORMANCE";

export function isRuntimeFlagEnabled(value: string | undefined): boolean {
  return value === "true" || value === "1";
}

export function installRuntimeFlags(
  env: NodeJS.ProcessEnv = process.env,
  globalObj: RuntimeFlagsGlobal = globalThis as RuntimeFlagsGlobal,
): void {
  if (!("DEBUG" in globalObj)) {
    globalObj.DEBUG = isRuntimeFlagEnabled(env.DEBUG);
  }

  if (!("CONFORMANCE" in globalObj)) {
    globalObj.CONFORMANCE = isRuntimeFlagEnabled(env.CONFORMANCE);
  }

  installRuntimeFlagBindings(["DEBUG", "CONFORMANCE"], globalObj);
}

export function installRuntimeFlagBindings(
  flagNames: readonly RuntimeFlagName[],
  globalObj: RuntimeFlagsGlobal = globalThis as RuntimeFlagsGlobal,
): void {
  const globalEval = typeof globalObj.eval === "function" ? globalObj.eval.bind(globalObj) : null;
  if (globalEval === null || flagNames.length === 0) return;

  const declarations = flagNames.map(
    (flagName) => `var ${flagName} = globalThis.${flagName} === true;`,
  );

  globalEval(declarations.join("\n"));
}

installRuntimeFlags();
