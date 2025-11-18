import { join } from 'node:path';
import { defineConfig } from 'tsdown';

import type { Plugin } from 'rolldown';

const ASSERTS_PATH = join(import.meta.dirname, 'src-js/utils/asserts.ts');

const replaceAssertsPlugin = createReplaceAssertsPlugin();
const plugins = [replaceAssertsPlugin];

const config = defineConfig({
  entry: ['src-js/cli.ts', 'src-js/index.ts'],
  format: 'esm',
  platform: 'node',
  target: 'node20',
  outDir: 'dist',
  clean: true,
  unbundle: false,
  hash: false,
  external: [
    // External native bindings
    './oxlint.*.node',
    '@oxlint/*',
  ],
  fixedExtension: false,
  // Handle `__filename`. Needed to bundle `typescript` for token methods.
  shims: true,
  // At present only compress syntax.
  // Don't mangle identifiers or remove whitespace, so `dist` code remains somewhat readable.
  minify: {
    compress: { keepNames: { function: true, class: true } },
    mangle: false,
    codegen: { removeWhitespace: false },
  },
  dts: { resolve: true },
  attw: true,
  define: { DEBUG: 'false' },
  plugins,
  inputOptions: {
    // For `replaceAssertsPlugin`
    experimental: { nativeMagicString: true },
  },
});

// Create separate debug build with debug assertions enabled
const debugConfig = defineConfig({
  ...config,
  outDir: 'debug',
  define: { DEBUG: 'true' },
  plugins: plugins.filter((plugin) => plugin !== replaceAssertsPlugin),
});

export default [config, debugConfig];

/**
 * Create a plugin to remove imports of `assert*` functions from `src-js/utils/asserts.ts`,
 * and replace those imports with empty function declarations.
 *
 * ```ts
 * // Original code
 * import { assertIs, assertIsNonNull } from '../utils/asserts.ts';
 *
 * // After transform
 * function assertIs() {}
 * function assertIsNonNull() {}
 * ```
 *
 * Minifier can already remove all calls to these functions as dead code, but only if the functions are defined
 * in the same file as the call sites.
 *
 * Problem is that `asserts.ts` is imported by files which end up in all output chunks.
 * So without this transform, TSDown creates a shared chunk for `asserts.ts`. Minifier works chunk-by-chunk,
 * so can't see that these functions are no-ops, and doesn't remove the function calls.
 *
 * Inlining these functions in each file solves the problem, and minifier removes all trace of them.
 *
 * @returns Plugin
 */
function createReplaceAssertsPlugin(): Plugin {
  return {
    name: 'replace-asserts',
    transform: {
      // Only process TS files in `src-js` directory
      filter: { id: /\/src-js\/.+\.ts$/ },

      async handler(code, id, meta) {
        const magicString = meta.magicString!;
        const program = this.parse(code, { lang: 'ts' });

        stmts: for (const stmt of program.body) {
          if (stmt.type !== 'ImportDeclaration') continue;

          // Check if import is from `utils/asserts.ts`.
          // `endsWith` check is just a shortcut to avoid resolving the specifier to a full path for most imports.
          const source = stmt.source.value;
          if (!source.endsWith('/asserts.ts') && !source.endsWith('/asserts.js')) continue;
          // oxlint-disable-next-line no-await-in-loop
          const importedId = await this.resolve(source, id);
          if (importedId === null || importedId.id !== ASSERTS_PATH) continue;

          // Replace `import` statement with empty function declarations
          let functionsCode = '';
          for (const specifier of stmt.specifiers) {
            // Skip this `import` statement if it's a default or namespace import - can't handle those
            if (specifier.type !== 'ImportSpecifier') continue stmts;
            functionsCode += `function ${specifier.local.name}() {}\n`;
          }
          magicString.overwrite(stmt.start, stmt.end, functionsCode);
        }

        return { code: magicString };
      },
    },
  };
}
