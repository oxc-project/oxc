export default {
  meta: {
    name: "estree-check",
  },
  rules: {
    check: {
      create(context) {
        const visits = [];
        return {
          Program(program) {
            visits.push(program.type);
          },
          VariableDeclaration(decl) {
            visits.push(`${decl.type}: ${decl.kind}`);
          },
          'VariableDeclaration:exit'(decl) {
            visits.push(`${decl.type}:exit: ${decl.kind}`);
          },
          Identifier(ident) {
            visits.push(`${ident.type}: ${ident.name}`);
          },
          ObjectExpression(expr) {
            visits.push(expr.type);
          },
          ParenthesizedExpression(expr) {
            // Should not be visited - no `ParenthesizedExpression`s in AST in ESLint
            visits.push(expr.type);
          },
          BinaryExpression(expr) {
            visits.push(`${expr.type}: ${expr.operator}`);
          },
          Literal(lit) {
            visits.push(`${lit.type}: ${lit.value}`);
          },
          TSTypeAliasDeclaration(decl) {
            visits.push(decl.type);
          },
          TSStringKeyword(keyword) {
            visits.push(keyword.type);
          },
          'Program:exit'(program) {
            visits.push(`${program.type}:exit`);
            context.report({
              message: `Visited nodes: ${visits.map(v => `'${v}'`).join(', ')}`,
              node: program,
            });
          },
        };
      },
    },
  },
};
