import type { Node, Plugin, Rule } from '../../../dist/index.js';

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

// Purpose of this test fixture is to ensure that source text and AST are available in `after` hook
// via `context.sourceCode` when the AST is not traversed

const createOnceRule: Rule = {
  createOnce(context) {
    return {
      Program(_program) {},
      after() {
        context.report({
          message: 'after:\n' +
            `text: ${JSON.stringify(context.sourceCode.text)}\n` +
            `getText(): ${JSON.stringify(context.sourceCode.getText())}\n` +
            // @ts-ignore
            `ast: "${context.sourceCode.ast.body[0].declarations[0].id.name}"`,
          node: SPAN,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: 'source-code-plugin',
  },
  rules: {
    'create-once': createOnceRule,
  },
};

export default plugin;
