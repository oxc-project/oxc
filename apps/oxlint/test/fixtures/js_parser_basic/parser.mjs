// Toy custom parser, wrapping `@typescript-eslint/parser`'s `parseForESLint`.
//
// Renames every `TemplateLiteral` node to a custom `CustomTemplate` node type,
// to emulate a parser which produces non-standard AST node types
// (like Ember's `.gjs` / `.gts` parser producing `GlimmerTemplate` nodes).

import { parseForESLint } from "@typescript-eslint/parser";

export default {
  parseForESLint(code, options) {
    const { ast, scopeManager, visitorKeys, services } = parseForESLint(code, options);
    renameTemplateLiterals(ast);
    return {
      ast,
      scopeManager,
      visitorKeys: { ...visitorKeys, CustomTemplate: ["quasis", "expressions"] },
      services: { ...services, isToyParser: true },
    };
  },
};

function renameTemplateLiterals(node) {
  if (node.type === "TemplateLiteral") node.type = "CustomTemplate";

  for (const key of Object.keys(node)) {
    if (key === "parent" || key === "range" || key === "loc") continue;

    const child = node[key];
    if (Array.isArray(child)) {
      for (const element of child) {
        if (element !== null && typeof element === "object" && typeof element.type === "string") {
          renameTemplateLiterals(element);
        }
      }
    } else if (child !== null && typeof child === "object" && typeof child.type === "string") {
      renameTemplateLiterals(child);
    }
  }
}
