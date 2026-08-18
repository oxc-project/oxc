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
 * The import is rewritten to match, which both keeps the plain name in scope and leaves the two
 * mapped functions unreferenced for the minifier to remove.
 *
 * Only valid where `SOURCEMAPS` is `false`, which is where the config includes it.
 *
 * The checks afterwards fail the build rather than let a mismatch through. A rewrite which dropped
 * the wrong argument, or left a call to a name nothing imports, produces a printer that is broken
 * everywhere or subtly wrong everywhere, and nothing downstream names the plugin as the cause.
 */
const REWRITES = {
  // `write` takes the `last` category between the code and the node, `writeNoLast` does not
  writeWithMap: { arity: 4, plain: "write" },
  writeWithMapNoLast: { arity: 3, plain: "writeNoLast" },
  writeWithMapEnd: { arity: 4, plain: "write" },
  // A standalone mark has no non-sourcemap equivalent. `void 0` is removed by the minifier.
  markWithMap: { arity: 2, plain: null },
  markWithMapNoName: { arity: 2, plain: null },
  markWithMapAfter: { arity: 2, plain: null },
  markWithMapAtStartOffset: { arity: 3, plain: null },
} as const;

const WRITE_MODULE = "./write.ts";

type MappedName = keyof typeof REWRITES;

const isMapped = (name: string): name is MappedName => name in REWRITES;

const plugin: Plugin = {
  name: "unmap-writes",
  transform: {
    // Only process TS files in `src-js/print` directory
    filter: { id: /\/src-js\/print\/.+\.ts$/ },

    handler(code, path, meta) {
      const magicString = meta.magicString!;
      const { program, errors } = parseSync(path, code);
      if (errors.length !== 0) throw new Error(`Failed to parse ${path}: ${errors[0].message}`);

      // Plain names this file will need in scope once its calls are rewritten
      const needed = new Set<string>();
      let rewrote = false;

      new Visitor({
        CallExpression(node) {
          const { callee } = node;
          if (callee.type !== "Identifier" || !isMapped(callee.name)) return;

          const { arity, plain } = REWRITES[callee.name];
          const args = node.arguments;
          if (args.length !== arity) {
            throw new Error(
              `\`${callee.name}\` takes ${arity} arguments, found ${args.length}: ` +
                `${path}:${node.start}`,
            );
          }

          rewrote = true;
          if (plain === null) {
            magicString.overwrite(node.start, node.end, "void 0");
          } else {
            magicString.overwrite(callee.start, callee.end, plain);
            // Remove `, node`, from the end of the argument before it to the end of it
            magicString.remove(args[arity - 2].end, args[arity - 1].end);
            needed.add(plain);
          }
        },
      }).visit(program);

      if (rewrote) {
        // Rewrite the import to drop the mapped names and carry whatever the rewrites now need.
        // `write.ts` declares them itself and imports nothing, so it never reaches this.
        // A file may import from `write.ts` more than once - the types separately from the values.
        // The one to rewrite is whichever brought the mapped names in.
        const imports = program.body.filter(
          (statement) =>
            statement.type === "ImportDeclaration" &&
            statement.source.value === WRITE_MODULE &&
            statement.specifiers.some(
              (specifier) =>
                specifier.type === "ImportSpecifier" &&
                specifier.imported.type === "Identifier" &&
                isMapped(specifier.imported.name),
            ),
        );

        if (imports.length !== 1) {
          throw new Error(
            `Expected 1 \`${WRITE_MODULE}\` import bringing in mapped writes, found ` +
              `${imports.length}: ${path}`,
          );
        }

        const importNode = imports[0];
        if (importNode.type !== "ImportDeclaration") throw new Error(`Unreachable: ${path}`);

        const names = new Set(needed);
        for (const specifier of importNode.specifiers) {
          if (specifier.type !== "ImportSpecifier" || specifier.imported.type !== "Identifier") {
            throw new Error(`Unexpected \`${WRITE_MODULE}\` import specifier: ${path}`);
          }
          if (!isMapped(specifier.imported.name)) names.add(specifier.local.name);
        }

        const sorted = [...names].sort();
        magicString.overwrite(
          importNode.start,
          importNode.end,
          `import { ${sorted.join(", ")} } from "${WRITE_MODULE}";`,
        );
      }

      const transformed = magicString.toString();
      const { program: after, errors: afterErrors } = parseSync(path, transformed);
      if (afterErrors.length !== 0) {
        throw new Error(`Transform produced invalid code: ${path}: ${afterErrors[0].message}`);
      }

      // Nothing may still reach a mapped write.
      // Their declarations stay, for the minifier to remove once nothing imports them.
      const bound = new Set<string>();
      const problems: string[] = [];

      new Visitor({
        ImportSpecifier(node) {
          if (node.imported.type !== "Identifier") return;
          if (isMapped(node.imported.name)) {
            problems.push(`import of ${node.imported.name} at ${node.start}`);
          }
          bound.add(node.local.name);
        },
        FunctionDeclaration(node) {
          if (node.id !== null) bound.add(node.id.name);
        },
      }).visit(after);

      const called = new Set<string>();
      new Visitor({
        CallExpression(node) {
          const { callee } = node;
          if (callee.type !== "Identifier") return;
          if (isMapped(callee.name)) problems.push(`call to ${callee.name} at ${node.start}`);
          if (callee.name === "write" || callee.name === "writeNoLast") called.add(callee.name);
        },
      }).visit(after);

      // A rewritten call to a name this file does not have in scope would be a `ReferenceError`
      // on the first print, so check rather than assume the import rewrite above got it right
      for (const name of called) {
        if (!bound.has(name)) problems.push(`\`${name}\` is called but not in scope`);
      }

      if (problems.length !== 0) {
        throw new Error(`unmap-writes left ${path} broken: ${problems.join(", ")}`);
      }

      return { code: magicString };
    },
  },
};

export default plugin;
