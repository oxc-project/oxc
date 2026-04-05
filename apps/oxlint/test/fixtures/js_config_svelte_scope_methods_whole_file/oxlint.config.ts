import { defineConfig } from "#oxlint";

function createLoc(start: number, end: number) {
  return {
    start: { line: 1, column: start },
    end: { line: 1, column: end },
  };
}

const svelteStubParser = {
  parseForESLint(code: string) {
    const markupEnd = code.endsWith("\n") ? code.length - 1 : code.length;
    const identifier = {
      type: "Identifier",
      name: "externalGlobal",
      range: [0, 1],
      loc: createLoc(0, 1),
    };
    const expressionStatement = {
      type: "ExpressionStatement",
      expression: identifier,
      range: [0, 1],
      loc: createLoc(0, 1),
    };
    const program = {
      type: "Program",
      sourceType: "module",
      body: [expressionStatement],
      range: [0, markupEnd],
      loc: createLoc(0, markupEnd),
      comments: [],
      tokens: [],
    };

    const reference = {
      identifier,
      from: null as unknown,
      resolved: null as unknown,
      writeExpr: null,
      init: false,
      isWrite: () => false,
      isRead: () => true,
      isReadOnly: () => true,
      isWriteOnly: () => false,
      isReadWrite: () => false,
    };

    const globalScope = {
      type: "global",
      isStrict: false,
      upper: null,
      childScopes: [] as any[],
      variableScope: null as unknown,
      block: program,
      variables: [] as any[],
      set: new Map<string, any>(),
      references: [reference] as any[],
      through: [reference] as any[],
      functionExpressionScope: false,
      implicit: {
        variables: [] as any[],
        set: new Map<string, any>(),
      },
    };

    const moduleScope = {
      type: "module",
      isStrict: true,
      upper: globalScope,
      childScopes: [] as any[],
      variableScope: null as unknown,
      block: program,
      variables: [] as any[],
      set: new Map<string, any>(),
      references: [reference] as any[],
      through: [reference] as any[],
      functionExpressionScope: false,
    };

    const variable = {
      name: "externalGlobal",
      scope: moduleScope,
      identifiers: [identifier],
      references: [reference],
      defs: [],
    };

    reference.from = moduleScope;
    reference.resolved = variable;
    globalScope.childScopes.push(moduleScope);
    globalScope.variableScope = globalScope;
    moduleScope.variableScope = moduleScope;
    globalScope.variables.push(variable);
    moduleScope.variables.push(variable);
    globalScope.set.set(variable.name, variable);
    moduleScope.set.set(variable.name, variable);

    const scopeManager = {
      scopes: [globalScope, moduleScope],
      globalScope,
      acquire(node: unknown) {
        return node === program ? moduleScope : null;
      },
      getDeclaredVariables(node: unknown) {
        return node === program ? [variable] : [];
      },
    };

    return {
      ast: program,
      services: {
        isSvelte: true,
      },
      visitorKeys: {
        Program: ["body"],
        ExpressionStatement: ["expression"],
        Identifier: [],
      },
      scopeManager,
    };
  },
};

export default defineConfig({
  categories: {
    correctness: "off",
  },
  jsPlugins: ["./plugin.ts"],
  overrides: [
    {
      files: ["**/*.svelte"],
      languageOptions: {
        parser: svelteStubParser,
      },
      rules: {
        "whole-file-svelte-scope/methods": "error",
      },
    },
  ],
});
