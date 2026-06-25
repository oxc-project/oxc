// Test that Oxlint's `WeakMap` patch correctly isolates per-file state across multiple files.
//
// In ESLint, `context.sourceCode` is a different object for each file, so a `WeakMap` keyed by
// `context.sourceCode` naturally gives each file its own entry.
//
// In Oxlint, `context.sourceCode` is a singleton shared across all files and rules.
// Without the `WeakMap` patch (in `src-js/plugins/weak_map.ts`), data stored under the `sourceCode` key
// would leak between files. The patch intercepts `WeakMap` operations on the `sourceCode` singleton
// and resets the stored values between files (via `resetWeakMaps()` in `lint.ts`).
//
// This plugin creates 3 rules that share a `WeakMap<SourceCode, PerFileState>` cache - a common pattern
// in real ESLint plugins. Each rule records its ID in the shared per-file state when it visits an `Identifier`.
// In `Program:exit`, the plugin reports the collected state.
//
// If the patch works correctly, each file gets its own fresh state, and the report for each file
// shows that file's own filename and exactly 3 rule names and 3 identifier names.
//
// If the patch is broken (data leaks between files), the second file would reuse the first file's
// state. The `identNames` and `ruleNames` arrays would have 6 entries instead of 3.

import type { Plugin, Rule, SourceCode, Context } from "#oxlint/plugins";

interface PerFileState {
  filename: string;
  identNames: string[];
  ruleNames: string[];
}

const cache = new WeakMap<SourceCode, PerFileState>();

function getCachedData(context: Context): PerFileState {
  const cachedData = cache.get(context.sourceCode);
  if (cachedData !== undefined) return cachedData;

  const data: PerFileState = {
    filename: context.filename,
    identNames: [],
    ruleNames: [],
  };
  cache.set(context.sourceCode, data);
  return data;
}

function createRuleUsingCache(): Rule {
  return {
    create(context) {
      return {
        Identifier(node) {
          const data = getCachedData(context);
          data.identNames.push(node.name);
          data.ruleNames.push(context.id.slice(context.id.indexOf("/") + 1));
        },

        "Program:exit"(node) {
          const data = getCachedData(context);

          context.report({
            message:
              "Rules which have accessed this file:\n" +
              `filename: ${data.filename}\n` +
              `ident names: ${data.identNames.join(", ")}\n` +
              `rule names: ${data.ruleNames.sort().join(", ")}`,
            node,
          });
        },
      };
    },
  };
}

const plugin: Plugin = {
  meta: {
    name: "weakmap-cache-plugin",
  },
  rules: {
    cache1: createRuleUsingCache(),
    cache2: createRuleUsingCache(),
    cache3: createRuleUsingCache(),
  },
};

export default plugin;
