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
 * The mapped writes and `markWithMap*` exist to record a source mapping for the node they are given.
 * A build without source map support has nothing to record, so the call becomes the plain write
 * it would otherwise be, and the node argument goes with it - it would still be evaluated,
 * and held live across the call, for a function which ignores it.
 *
 * The import is rewritten to match, which both keeps the plain name in scope and leaves the mapped functions
 * unreferenced for the minifier to remove.
 *
 * Only valid where `SOURCEMAPS` is `false`, which is where the config includes it.
 *
 * The checks afterwards fail the build rather than let a mismatch through. A rewrite which dropped
 * the wrong argument, or left a call to a name nothing imports, produces a printer that is broken
 * everywhere or subtly wrong everywhere, and nothing downstream names the plugin as the cause.
 */
const REWRITES = {
  // `write` takes the `last` category between the code and the node, `writeNoLast` does not
  writeWithMap: { arity: 4, rename: "write" },
  writeWithMapNoLast: { arity: 3, rename: "writeNoLast" },
  writeWithMapEnd: { arity: 4, rename: "write" },
  // A standalone mark has no non-sourcemap equivalent. `void 0` is removed by the minifier.
  markWithMap: { arity: 2, rename: null },
  markWithMapNoName: { arity: 2, rename: null },
  markWithMapAfter: { arity: 2, rename: null },
  markWithMapAtStartOffset: { arity: 3, rename: null },
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
        // Remove last arg (`node`) from calls, and rename callees.
        // A call with no plain equivalent goes in full.
        CallExpression(node) {
          const { callee } = node;
          if (callee.type !== "Identifier") return;

          const name = callee.name as MappedName;
          const rewrite = REWRITES[name];
          if (rewrite === undefined) return;

          const { arity, rename } = rewrite;
          const args = node.arguments;
          if (args.length !== arity) {
            throw new Error(
              `\`${callee.name}\` takes ${arity} arguments, found ${args.length}: `
                + `${path}:${node.start}`,
            );
          }

          if (rename === null) {
            magicString.overwrite(node.start, node.end, "void 0");
          } else {
            // Remove `, node`, from the end of the argument before it to the end of it
            magicString.remove(args.at(-2)!.end, args.at(-1)!.end);

            // Rename callee, and record that it needs to be imported under new name
            magicString.overwrite(callee.start, callee.end, rename);
            neededImportReplacements.add(name);
          }
        },

        // Check `arity` is correct
        FunctionDeclaration(node) {
          const { id } = node;
          if (id === null) return;

          const name = id.name as MappedName;
          const rewrite = REWRITES[name];
          if (rewrite === undefined) return;

          const { arity } = rewrite;
          const { params } = node;
          if (params.length !== arity) {
            throw new Error(
              `\`${id.name}\` defined with ${params.length} parameters, expected ${arity}: `
                + `${path}:${node.start}`,
            );
          }
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
              // A name whose calls become `void 0` has nothing to import in its place
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
