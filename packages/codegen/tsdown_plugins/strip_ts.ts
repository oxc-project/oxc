import type { Plugin } from "rolldown";

/**
 * Plugin to remove the TypeScript-only regions of `print.ts` from the JS-only builds.
 *
 * The `TS` build-time constant (see `src-js/globals.d.ts`) lets the minifier drop TS-only code
 * in expression positions - `if (TS && node.declare)` folds to nothing when `TS` is `false`.
 * But `case "TSEnumDeclaration":` switch arms are not expressions, and no minifier can remove an arm
 * on the grounds that its node type never occurs. Those arms are fenced with `IF TS` / `END_IF`
 * block-comment markers (the same convention as the raw transfer deserializer builds):
 *
 * ```ts
 * /* IF TS *\/
 * case "TSEnumDeclaration":
 *   ...
 *   break;
 * /* END_IF *\/
 * ```
 *
 * (`*\/` escapes above only because this is itself a block comment.)
 *
 * This plugin deletes everything from each `IF TS` marker line through its `END_IF` marker line.
 * Once the arms are gone, the TS-only printer functions they called lose their last references,
 * and tree-shaking removes them wholesale.
 *
 * It's a plain text transform (the fences are only visible in source text), so it must run before
 * the AST-based plugins in the plugin list, and only in builds where `TS` is `false`.
 */
const REGION = /[ \t]*\/\* IF TS \*\/\n[\s\S]*?\/\* END_IF \*\/\n/g;

// A factory, so each build counts its own regions rather than sharing a total with the others
const stripTsPlugin = (): Plugin => {
  let strippedRegions = 0;

  return {
    name: "strip-ts",

    buildStart() {
      strippedRegions = 0;
    },

    transform: {
      // Only process TS files in `src-js/print` directory
      filter: { id: /\/src-js\/print\/.+\.ts$/ },

      handler(code, _path, meta) {
        const magicString = meta.magicString!;
        for (const match of code.matchAll(REGION)) {
          strippedRegions++;
          const start = match.index!;
          magicString.remove(start, start + match[0].length);
        }

        const transformed = magicString.toString();

        // Unbalanced or nested fences leave a marker behind.
        // Most files have no fences at all, so the "did this plugin do anything" check is per build, in `buildEnd`.
        if (transformed.includes("IF TS") || transformed.includes("END_IF")) {
          throw new Error("strip-ts: unbalanced or nested `/* IF TS */` / `/* END_IF */` fences");
        }

        return { code: magicString };
      },
    },

    buildEnd() {
      // No fences anywhere means they have been lost and this plugin is silently doing nothing
      if (strippedRegions === 0) {
        throw new Error("strip-ts: no `/* IF TS */` / `/* END_IF */` fences found");
      }
    },
  };
};

export default stripTsPlugin;
