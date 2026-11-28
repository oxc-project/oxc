// Test the source type that JS plugins get for files without ESM syntax.
//
// Oxc parses `.js`, `.jsx`, `.ts`, and `.tsx` files, and Astro scripts, as "unambiguous".
// If the code has no ESM syntax (`import`, `export`, `import.meta`, or a top-level `await` expression),
// the parser makes it a script.
// ESLint's flat config parses `.js` files as modules by default, also if they have no ESM syntax,
// and `.cjs` files as CommonJS. JS plugins must get the same source type as in ESLint.
//
// The `FunctionDeclaration` visitor does the same as the `no-inner-declarations` rule of eslint-plugin-svelte:
// it wraps the node in a `Proxy`, and calls `getScope` with the proxy.
// The scope manager does not know the proxy, so `getScope` returns the innermost scope of `Program`.
// In a module, this is the module scope, and its `upper` is the global scope.
// In a script, this is the global scope, and its `upper` is `null`.
//
// Rust rules must still see a script. `.oxlintrc.json` enables the native `no-implicit-globals` rule
// for `without_esm.js`. This rule reports top-level declarations only in a script.

import type { Plugin, Scope } from "#oxlint/plugins";

function scopeType(scope: Scope | null): string {
  return scope === null ? "null" : scope.type;
}

const plugin: Plugin = {
  meta: {
    name: "source-type-plugin",
  },
  rules: {
    check: {
      create(context) {
        return {
          Program(program) {
            const { scopeManager } = context.sourceCode;
            context.report({
              message:
                `Program.sourceType: ${program.sourceType}\n`
                + `languageOptions.sourceType: ${context.languageOptions.sourceType}\n`
                + `scopes: ${scopeManager.scopes.map((scope) => scope.type).join(", ")}`,
              node: program,
            });
          },
          FunctionDeclaration(node) {
            const scope = context.sourceCode.getScope(node);
            const proxyScope = context.sourceCode.getScope(new Proxy(node, {}));
            context.report({
              message:
                `getScope(node): ${scope.type}, isStrict: ${scope.isStrict}\n`
                + `getScope(proxy): ${proxyScope.type}, upper: ${scopeType(proxyScope.upper)}`,
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
