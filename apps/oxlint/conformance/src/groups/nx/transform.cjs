const { rootDir } = require("./config.cjs");
const ts = require(require.resolve("typescript", { paths: [rootDir] }));

module.exports = {
  process(source, filename) {
    // Nx uses Jest's hoisted module mocks. Preserve their order before imports.
    const file = ts.createSourceFile(filename, source, ts.ScriptTarget.Latest, true);
    const mocks = [];
    const statements = [];
    for (const statement of file.statements) {
      if (
        ts.isExpressionStatement(statement)
        && ts.isCallExpression(statement.expression)
        && statement.expression.expression.getText(file) === "jest.mock"
      ) {
        mocks.push(statement);
      } else {
        statements.push(statement);
      }
    }
    const reordered = ts.factory.updateSourceFile(file, [...mocks, ...statements]);
    return {
      code: ts.transpileModule(ts.createPrinter().printFile(reordered), {
        fileName: filename,
        compilerOptions: {
          module: ts.ModuleKind.CommonJS,
          target: ts.ScriptTarget.ES2022,
          esModuleInterop: true,
        },
      }).outputText,
    };
  },
};
