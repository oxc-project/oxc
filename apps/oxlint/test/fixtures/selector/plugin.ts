import type { ESTree, Plugin, Visitor } from "#oxlint/plugin";

const plugin: Plugin = {
  meta: {
    name: "selectors",
  },
  rules: {
    check: {
      create(context) {
        const keys = [
          // Wildcard
          "*",
          // Node type
          "Identifier",
          // Attributes
          "Identifier[name=a]",
          "Identifier[name=d]",
          // :matches
          ":matches(Identifier, FunctionDeclaration)",
          ":matches(Identifier[name=a], FunctionDeclaration[id.name=foo])",
          // :not
          ":not(Identifier)",
          // class
          ":statement",
          ":Declaration",
          ":paTTern",
          ":expression",
          ":FUNCTION",
          ":function > Identifier",
          // Child
          "Property > Identifier",
          "ObjectExpression > Identifier", // does not match
          "ObjectExpression > Property > Identifier",
          "ObjectExpression > Property > Identifier[name=a]",
          "ObjectExpression > Property > Identifier[name=b]", // does not match
          "ArrayExpression > Identifier",
          "ArrayExpression > Identifier[name=c]",
          "Program > VariableDeclaration > VariableDeclarator > ObjectExpression > Property > ArrayExpression > Identifier",
          "Program > FunctionDeclaration",
          // Descendent
          "ObjectExpression Identifier",
          "ObjectExpression ArrayExpression",
          "ArrayExpression Identifier",
          "ArrayExpression Identifier[name=b]",
          // Sibling
          "Identifier ~ Identifier",
          "Property ~ [type]",
          "VariableDeclaration ~ FunctionDeclaration",
          "VariableDeclaration + FunctionDeclaration",
          // Combination
          ":matches(ObjectExpression > SpreadElement, FunctionDeclaration)",
          ":matches(ObjectExpression > SpreadElement, FunctionDeclaration[id.name=bar])",
          // Wildcard
          "*:exit",
        ];

        const visitor: Visitor = {};
        const visits: { key: string; node: ESTree.Node }[] = [];

        for (const key of keys) {
          visitor[key] = (node) => {
            visits.push({ key, node });
          };
        }

        visitor["Program:exit"] = (program) => {
          const visitLog = visits
            .map(({ key, node }) => {
              const { type } = node;
              let nodeDescription = type;
              if (type === "Identifier") {
                nodeDescription += `(${node.name})`;
              } else if (type === "FunctionDeclaration") {
                nodeDescription += `(${node.id?.name})`;
              }
              return `${key}: ${nodeDescription}`;
            })
            .join("\n");

          context.report({
            message: `\n${visitLog}`,
            node: program,
          });
        };

        return visitor;
      },
    },
  },
};

export default plugin;
