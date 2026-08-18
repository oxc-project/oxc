import { parseSync, Visitor } from "oxc-parser";

import type { Plugin } from "rolldown";

/**
 * Plugin to turn every mapped write back into a plain one, for builds without source maps.
 *
 * ```ts
 * // Original code
 * writeWithMap(state, "declare ", CAT_OTHER, node);
 *
 * // After transform
 * write(state, "declare ", CAT_OTHER);
 * ```
 *
 * The mapped writes and `markMap*` exist to record a source mapping for the node they are given.
 * A build without source map support has nothing to record, so the call becomes the plain write
 * it would otherwise be, and the node argument goes with it - it would still be evaluated,
 * and held live across the call, for a function which ignores it.
 *
 * `remove` is how many trailing arguments are dropped from the call.
 *
 * The import is rewritten to match, which both keeps the plain name in scope and leaves the mapped functions
 * unreferenced for the minifier to remove.
 *
 * `printString` and `printNonNegativeFloat` take a node only to hand it on to their mapped writes.
 * So they are rewritten the same way, but keep their names - the argument comes off every call.
 * The parameter also comes off their declarations which, unlike the mapped writes, survive into the build.
 *
 * Only valid where `SOURCEMAPS` is `false`, which is where the config includes it.
 *
 * The checks afterwards fail the build rather than let a mismatch through. A rewrite which dropped
 * the wrong argument, or left a call to a name nothing imports, produces a printer that is broken
 * everywhere or subtly wrong everywhere, and nothing downstream names the plugin as the cause.
 */
const REWRITES = {
  // `write` takes the `last` category between the code and the node, `writeNoLast` does not
  writeWithMap: { arity: 4, remove: 1, rename: "write" },
  writeWithMapNamed: { arity: 4, remove: 1, rename: "write" },
  writeWithMapNoLast: { arity: 3, remove: 1, rename: "writeNoLast" },
  writeWithMapNamedNoLast: { arity: 3, remove: 1, rename: "writeNoLast" },
  writeWithMapEnd: { arity: 4, remove: 1, rename: "write" },
  // `rename: null` because a standalone mark has no non-sourcemap equivalent
  markMapStart: { arity: 2, remove: 1, rename: null },
  markMapAfter: { arity: 2, remove: 1, rename: null },
  markMapAtStartOffset: { arity: 3, remove: 2, rename: null },
  // `rename: null` to transform the function declarations, removing the `node` param
  printString: { arity: 3, remove: 1, rename: null },
  printNonNegativeFloat: { arity: 3, remove: 1, rename: null },
} as const;

const WRITE_MODULE = "./write.ts";

type MappedName = keyof typeof REWRITES;

const plugin: Plugin = {
  name: "unmap-writes",
  transform: {
    // Only process TS files in `src-js/print` directory
    filter: { id: /\/src-js\/print\/.+\.ts$/ },

    handler(code, path, meta) {
      // Parse file
      const magicString = meta.magicString!;
      const { program, errors } = parseSync(path, code);
      if (errors.length !== 0) throw new Error(`Failed to parse ${path}: ${errors[0].message}`);

      // Imports from `write.ts` which have been replaced, and which need to be replaced.
      // The 2 are compared and ensured equal at the end of the visitation pass.
      const importReplacements: string[] = [];
      const neededImportReplacements = new Set<string>();
      let importIsTransformed = false;

      new Visitor({
        // Remove the dropped args from calls, and rename callees where required
        CallExpression(node) {
          const { callee } = node;
          if (callee.type !== "Identifier") return;

          const name = callee.name as MappedName;
          const rewrite = REWRITES[name];
          if (rewrite === undefined) return;

          const { arity, remove, rename } = rewrite;
          const args = node.arguments;
          if (args.length !== arity) {
            throw new Error(
              `\`${callee.name}\` takes ${arity} arguments, found ${args.length}: `
                + `${path}:${node.start}`,
            );
          }

          // Remove the dropped arguments, from the end of the last kept one to the end of the last
          magicString.remove(args[arity - remove - 1].end, args[arity - 1].end);

          // If callee needs to be renamed, rename it, and record that it needs to be imported under new name
          if (rename !== null) {
            magicString.overwrite(callee.start, callee.end, rename);
            neededImportReplacements.add(name);
          }
        },

        // Remove the dropped params from declarations
        FunctionDeclaration(node) {
          const { id } = node;
          if (id === null) return;

          const name = id.name as MappedName;
          const rewrite = REWRITES[name];
          if (rewrite === undefined) return;

          const { arity, remove } = rewrite;
          const { params } = node;
          if (params.length !== arity) {
            throw new Error(
              `\`${id.name}\` defined with ${params.length} parameters, expected ${arity}: `
                + `${path}:${node.start}`,
            );
          }

          // Remove the dropped params, from the end of the last kept one to the end of the last
          magicString.remove(params[arity - remove - 1].end, params[arity - 1].end);
        },

        // Replace import of functions whose calls are renamed or removed
        ImportDeclaration(node) {
          // Skip imports not from `write.ts`, and type imports
          if (node.source.value !== WRITE_MODULE || node.importKind === "type") return;

          // For simplicity, we only support a single import from `write.ts` (not including type imports)
          if (importIsTransformed) {
            throw new Error(`Multiple imports from \`${WRITE_MODULE}\`: ${path}`);
          }
          importIsTransformed = true;

          // Collect all specifiers, replacing any that need to be
          const specifierNames = new Set<string>();
          for (const specifier of node.specifiers) {
            if (
              specifier.type !== "ImportSpecifier"
              || specifier.importKind === "type"
              || specifier.imported.type !== "Identifier"
            ) {
              throw new Error(
                `Only simple imports from \`${WRITE_MODULE}\` are supported: ${path}`,
              );
            }

            const name = specifier.imported.name as MappedName;
            const rewrite = REWRITES[name];
            if (rewrite !== undefined) {
              const { rename } = rewrite;
              if (rename !== null) {
                specifierNames.add(rename);
                importReplacements.push(name);
                continue;
              }
            }

            specifierNames.add(name);
          }

          if (importReplacements.length > 0) {
            magicString.overwrite(
              node.start,
              node.end,
              `import { ${[...specifierNames].join(", ")} } from "${WRITE_MODULE}";`,
            );
          }
        },
      }).visit(program);

      // Check that replaced callees also had the corresponding import replaced
      if (importReplacements.length !== neededImportReplacements.size) {
        const missing = [];
        for (const name of neededImportReplacements) {
          if (!importReplacements.includes(name)) missing.push(name);
        }

        if (missing.length > 0) {
          throw new Error(
            `Missing import of ${missing.map((name) => `\`${name}\``).join(", ")}: ${path}`,
          );
        }
      }

      return { code: magicString };
    },
  },
};

export default plugin;
