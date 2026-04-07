import type { Node, Plugin, Rule, Scope } from "#oxlint/plugins";

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const rule: Rule = {
  create(context) {
    const { sourceCode } = context;

    return {
      Program(node) {
        // Use scopes[1] which is the module scope (ESM) or outer function scope (CJS).
        const topLevelScope = sourceCode.scopeManager.scopes[1];

        // 1. Mark `unusedTopLevel` from `Program` (tests module scope adjustment).
        //    Check `eslintUsed` before and after to verify it's being set.
        const beforeUnused = getEslintUsed(topLevelScope, "unusedTopLevel");
        const markUnused = sourceCode.markVariableAsUsed("unusedTopLevel", node);
        const afterUnused = getEslintUsed(topLevelScope, "unusedTopLevel");

        context.report({
          message:
            "[1] mark `unusedTopLevel` from Program:\n" +
            `before: ${beforeUnused}\n` +
            `result: ${markUnused}\n` +
            `after: ${afterUnused}`,
          node: SPAN,
        });

        // 2. Non-existent variable returns `false`.
        context.report({
          message:
            "[2] mark `nonExistent` from Program:\n" +
            `result: ${sourceCode.markVariableAsUsed("nonExistent", node)}`,
          node: SPAN,
        });

        // 3. Call without 2nd arg - defaults to Program node.
        //    Should find the module-level `shadowedName`, NOT `inner`'s local one.
        const beforeShadowed = getEslintUsed(topLevelScope, "shadowedName");
        const markShadowed = sourceCode.markVariableAsUsed("shadowedName");
        const afterShadowed = getEslintUsed(topLevelScope, "shadowedName");

        context.report({
          message:
            "[3] mark `shadowedName` (no refNode):\n" +
            `before: ${beforeShadowed}\n` +
            `result: ${markShadowed}\n` +
            `after: ${afterShadowed}`,
          node: SPAN,
        });
      },

      "FunctionDeclaration[id.name='outer']"(node) {
        const outerScope = sourceCode.getScope(node);

        // 4. Mark a variable in the current function scope.
        const beforeNested = getEslintUsed(outerScope, "nestedVar");
        const markNested = sourceCode.markVariableAsUsed("nestedVar", node);
        const afterNested = getEslintUsed(outerScope, "nestedVar");

        context.report({
          message:
            "[4] mark `nestedVar` from outer:\n" +
            `before: ${beforeNested}\n` +
            `result: ${markNested}\n` +
            `after: ${afterNested}`,
          node: SPAN,
        });

        // 5. Walk up from `outer` to module scope to find `unusedTopLevel2`.
        const moduleScope = outerScope.upper!;
        const beforeTop2 = getEslintUsed(moduleScope, "unusedTopLevel2");
        const markTop2 = sourceCode.markVariableAsUsed("unusedTopLevel2", node);
        const afterTop2 = getEslintUsed(moduleScope, "unusedTopLevel2");

        context.report({
          message:
            "[5] mark `unusedTopLevel2` from outer:\n" +
            `before: ${beforeTop2}\n` +
            `result: ${markTop2}\n` +
            `after: ${afterTop2}`,
          node: SPAN,
        });
      },

      "FunctionDeclaration[id.name='inner']"(node) {
        const innerScope = sourceCode.getScope(node);
        const outerScope = innerScope.upper!;

        // 6. Walk up from `inner` through `outer` to find `nestedVar2` in parent scope.
        const beforeNested = getEslintUsed(outerScope, "nestedVar2");
        const markNested = sourceCode.markVariableAsUsed("nestedVar2", node);
        const afterNested = getEslintUsed(outerScope, "nestedVar2");

        context.report({
          message:
            "[6] mark `nestedVar2` from inner:\n" +
            `before: ${beforeNested}\n` +
            `result: ${markNested}\n` +
            `after: ${afterNested}`,
          node: SPAN,
        });

        // 7. Non-existent variable from nested scope returns `false`.
        context.report({
          message:
            "[7] mark `doesNotExist` from inner:\n" +
            `result: ${sourceCode.markVariableAsUsed("doesNotExist", node)}`,
          node: SPAN,
        });
      },
    };
  },
};

/**
 * Helper to read `eslintUsed` from a variable found by name in a scope.
 */
function getEslintUsed(scope: Scope, name: string): boolean | undefined {
  const variable = scope.set.get(name);
  // @ts-expect-error - `eslintUsed` isn't part of public API
  return variable?.eslintUsed;
}

const plugin: Plugin = {
  meta: { name: "mark-used-plugin" },
  rules: { "mark-used": rule },
};

export default plugin;
