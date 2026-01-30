import assert from "node:assert";

import type { Node, Plugin, Rule, Scope } from "#oxlint/plugin";

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
    let moduleScope: Scope | null = null;

    return {
      Program(program) {
        const { scopeManager } = context.sourceCode;

        moduleScope = scopeManager.scopes.at(1) as unknown as Scope;
        assert.equal(moduleScope.upper, scopeManager.globalScope);

        context.report({
          message:
            `File has ${scopeManager.scopes.length} scopes:\n- ` +
            scopeManager.scopes
              .map((s: any) => {
                const name = s.block?.id?.name;
                return name ? `${s.type}(${name})` : `${s.type}`;
              })
              .join("\n- "),
          node: SPAN,
        });

        const acquiredScope = scopeManager.acquire(program);
        assert.equal(acquiredScope, scopeManager.globalScope);
      },
      VariableDeclaration(node) {
        if (node.declarations[0].id.type === "ObjectPattern") {
          const variables = context.sourceCode.scopeManager.getDeclaredVariables(node);
          context.report({
            message: `VariableDeclaration declares ${variables.length} variables: ${variables
              .map((v) => v.name)
              .join(", ")}.`,
            node: node,
          });
        }
      },
      FunctionDeclaration(node) {
        if (node.id && node.id.name === "topLevelFunction") {
          const topLevelFunctionScope = context.sourceCode.scopeManager.acquire(node)!;
          assert.equal(topLevelFunctionScope.upper, moduleScope);
          context.report({
            message: `topLevelFunction has ${topLevelFunctionScope.variables.length} local variables: ${topLevelFunctionScope?.variables
              .map((v) => v.name)
              .join(", ")}. Child scopes: ${topLevelFunctionScope.childScopes.length}.`,
            node: topLevelFunctionScope.block,
          });
        }
      },
      TSModuleDeclaration(node) {
        if (node.id.type === "Identifier" && node.id.name === "TopLevelModule") {
          const topLevelModuleScope = context.sourceCode.scopeManager.acquire(node)!;
          assert.equal(topLevelModuleScope.upper, moduleScope);
          context.report({
            message: `TopLevelModule has ${topLevelModuleScope.variables.length} local variables: ${topLevelModuleScope?.variables
              .map((v) => v.name)
              .join(", ")}. Child scopes: ${topLevelModuleScope.childScopes.length}.`,
            node: topLevelModuleScope.block,
          });
        }
      },
      StaticBlock(node) {
        const staticBlockScope = context.sourceCode.scopeManager.acquire(node)!;
        const upperBlock = staticBlockScope.upper!.block;
        assert("type" in upperBlock);
        assert(upperBlock.type === "ClassDeclaration");
        assert("id" in upperBlock);
        assert(typeof upperBlock.id === "object" && upperBlock.id !== null);
        assert("name" in upperBlock.id);
        assert.equal(upperBlock.id.name, "TestClass");
        context.report({
          message: `TestClass static block has ${staticBlockScope.variables.length} local variables: ${staticBlockScope?.variables
            .map((v) => v.name)
            .join(", ")}. Child scopes: ${staticBlockScope.childScopes.length}.`,
          node: node,
        });
      },
      LabeledStatement(node) {
        const labeledStatementScope = context.sourceCode.scopeManager.acquire(node.body)!;
        assert.equal(labeledStatementScope.upper, moduleScope);
        context.report({
          message: `LabeledStatement's block has ${labeledStatementScope.variables.length} local variables: ${labeledStatementScope?.variables
            .map((v) => v.name)
            .join(", ")}. Child scopes: ${labeledStatementScope.childScopes.length}.`,
          node: node,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "scope-manager-plugin",
  },
  rules: {
    scope: rule,
  },
};

export default plugin;
