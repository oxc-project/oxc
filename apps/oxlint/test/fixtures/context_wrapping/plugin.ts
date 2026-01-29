import type { Plugin, Rule, Context, Diagnostic, Node } from "#oxlint/plugin";

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

function createWrappedReportFunction(
  context: Context,
  prefix: string,
): (diagnostic: Diagnostic) => void {
  const { report } = context;
  return (diagnostic: Diagnostic) =>
    report({ ...diagnostic, message: `${prefix}: ${diagnostic.message}` });
}

const baseRule: Rule = {
  create(context) {
    context.report({ message: `id: ${context.id}`, node: SPAN });
    context.report({ message: `filename: ${context.filename}`, node: SPAN });
    context.report({
      message: `source text: ${JSON.stringify(context.sourceCode.text)}`,
      node: SPAN,
    });

    return {
      Identifier(node) {
        context.report({ message: `Identifier: '${node.name}'`, node });
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "wrapped-context",
  },
  rules: {
    // Two different forms of context wrapping
    "wrapped-rule": {
      create(context) {
        const wrappedContext = Object.create(context, {
          report: {
            value: createWrappedReportFunction(context, "wrapped 1"),
            writable: false,
          },
        });

        return baseRule.create(wrappedContext);
      },
    },
    "wrapped-rule2": {
      create(context) {
        const wrappedContext = Object.create(Object.getPrototypeOf(context));
        Object.assign(wrappedContext, context);
        wrappedContext.report = createWrappedReportFunction(context, "wrapped 2");

        return baseRule.create(wrappedContext);
      },
    },
  },
};

export default plugin;
