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

// Purpose of this test fixture is to ensure that parser services from parseForESLint
// are accessible via context.sourceCode.parserServices

const rule: Rule = {
  createOnce(context) {
    return {
      Program() {
        const parserServices = context.sourceCode.parserServices;
        
        // TypeScript ESLint parser provides program and esTreeNodeToTSNodeMap
        const hasProgram = 'program' in parserServices;
        const hasEsTreeNodeToTSNodeMap = 'esTreeNodeToTSNodeMap' in parserServices;
        
        context.report({
          message: `parserServices:\n` +
            `has program: ${hasProgram}\n` +
            `has esTreeNodeToTSNodeMap: ${hasEsTreeNodeToTSNodeMap}\n` +
            `keys: ${Object.keys(parserServices).join(', ')}`,
          node: SPAN,
        });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: 'parser-services-test-plugin',
  },
  rules: {
    'test-parser-services': rule,
  },
};

export default plugin;

