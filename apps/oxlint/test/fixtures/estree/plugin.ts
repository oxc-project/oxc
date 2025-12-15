// oxlint-disable typescript/restrict-template-expressions

import assert from "node:assert";

import type { Plugin } from "#oxlint";

const plugin: Plugin = {
  meta: {
    name: "estree-check",
  },
  rules: {
    check: {
      create(context) {
        // Note: Collect visits in an array instead of `context.report` in each visitor function,
        // to ensure visitation happens in right order.
        // Diagnostics may be output in different order from the order they're created in.
        const visits: string[] = [];
        return {
          Program(program) {
            context.report({
              message:
                "program:\n" +
                `start/end: [${program.start},${program.end}]\n` +
                `range: [${program.range}]\n` +
                `loc: [${JSON.stringify(program.loc)}]`,
              node: program,
            });
            visits.push(program.type);
          },
          VariableDeclaration(decl) {
            visits.push(`${decl.type}: ${decl.kind}`);
          },
          "VariableDeclaration:exit"(decl) {
            visits.push(`${decl.type}:exit: ${decl.kind}`);
          },
          VariableDeclarator(decl) {
            // `init` should not be `ParenthesizedExpression`
            visits.push(`${decl.type}: (init: ${decl.init?.type})`);
          },
          Identifier(ident) {
            // Check `loc` property returns same object each time it's accessed
            const { loc } = ident;
            const loc2 = ident.loc;
            assert(loc2 === loc);

            context.report({
              message:
                `ident "${ident.name}":\n` +
                `start/end: [${ident.start},${ident.end}]\n` +
                `range: [${ident.range}]\n` +
                `loc: [${JSON.stringify(loc)}]`,
              node: ident,
            });
            visits.push(`${ident.type}: ${ident.name}`);
          },
          ObjectExpression(expr) {
            visits.push(expr.type);
          },
          ParenthesizedExpression(paren) {
            // Should not be visited - no `ParenthesizedExpression`s in AST in ESLint
            visits.push(paren.type);
          },
          BinaryExpression(expr) {
            // `right` should not be `ParenthesizedExpression`
            visits.push(`${expr.type}: ${expr.operator} (right: ${expr.right.type})`);
          },
          Literal(lit) {
            visits.push(`${lit.type}: ${lit.value}`);
          },
          TSTypeAliasDeclaration(decl) {
            // `typeAnnotation` should not be `TSParenthesizedType`
            visits.push(`${decl.type}: (typeAnnotation: ${decl.typeAnnotation.type})`);
          },
          "TSTypeAliasDeclaration:exit"(decl) {
            // `typeAnnotation` should not be `TSParenthesizedType`
            visits.push(`${decl.type}:exit: (typeAnnotation: ${decl.typeAnnotation.type})`);
          },
          TSStringKeyword(keyword) {
            visits.push(keyword.type);
          },
          TSParenthesizedType(paren) {
            // Should not be visited - no `TSParenthesizedType`s in AST in TS-ESLint
            visits.push(paren.type);
          },
          TSUnionType(union) {
            // `types` should not be `TSParenthesizedType`
            visits.push(`${union.type}: (types: ${union.types.map((t) => t.type).join(", ")})`);
          },
          "TSUnionType:exit"(union) {
            // `types` should not be `TSParenthesizedType`
            visits.push(
              `${union.type}:exit: (types: ${union.types.map((t) => t.type).join(", ")})`,
            );
          },
          TSNumberKeyword(keyword) {
            visits.push(keyword.type);
          },
          "Program:exit"(program) {
            visits.push(`${program.type}:exit`);
            context.report({
              message: `Visited nodes:\n* ${visits.join("\n* ")}`,
              node: program,
            });
          },
        };
      },
    },
  },
};

export default plugin;
