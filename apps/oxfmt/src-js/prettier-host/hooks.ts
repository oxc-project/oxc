// ESM resolver hooks, registered by `libs/plugin-resolution.ts`.
//
// Runs on Node's module customization thread, so it shares no state with the
// rest of the process: the specifier map arrives through `initialize()`.

type Data = { map: Record<string, string> };
type ResolveContext = { conditions: string[]; importAttributes: object; parentURL?: string };
type ResolveResult = { url: string; shortCircuit?: boolean; format?: string };
type NextResolve = (specifier: string, context: ResolveContext) => ResolveResult;

let map: Record<string, string> = Object.create(null);

export function initialize(data: Data): void {
  // Copied onto a null-prototype object: the map crosses a thread boundary as a
  // plain object, and lookups use arbitrary module specifiers.
  map = Object.assign(Object.create(null), data.map);
}

export function resolve(
  specifier: string,
  context: ResolveContext,
  nextResolve: NextResolve,
): ResolveResult {
  const redirect = map[specifier];
  // `map` holds `file:` URLs already, so this never re-enters resolution.
  // Resolving a path instead would pick Prettier's `require` condition and hand
  // an ESM importer the CJS entry, which fails on the named exports.
  if (redirect !== undefined) return { url: redirect, shortCircuit: true };
  return nextResolve(specifier, context);
}
