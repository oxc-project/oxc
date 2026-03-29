import type { Plugin, Rule, ESTree } from "#oxlint/plugins";

type Node = ESTree.Node;

// The output of this test looked wrong to me (@overlookmotel).
// `onCodePathSegmentEnd` gets called before `onCodePathSegmentStart` for `VariableDeclaration`.
// But have run the same test in ESLint, and it produces exactly the same output. Weird!

const rule: Rule = {
  // @ts-expect-error - TODO: Make the types for CFG events work
  create(context) {
    const events: [string, string][] = [];

    return {
      onCodePathStart(codePath: any, node: Node) {
        events.push(["onCodePathStart", node.type]);
      },
      onCodePathEnd(codePath: any, node: Node) {
        events.push(["onCodePathEnd", node.type]);

        if (node.type === "Program") {
          context.report({
            message: `Visited nodes:\n* ${events.map(([eventName, type]) => `${eventName.padEnd(35)} ${type}`).join("\n* ")}`,
            node,
          });
        }
      },
      onCodePathSegmentStart(segment: any, node: Node) {
        events.push(["onCodePathSegmentStart", node.type]);
      },
      onCodePathSegmentEnd(segment: any, node: Node) {
        events.push(["onCodePathSegmentEnd", node.type]);
      },
      onUnreachableCodePathSegmentStart(segment: any, node: Node) {
        events.push(["onUnreachableCodePathSegmentStart", node.type]);
      },
      onUnreachableCodePathSegmentEnd(segment: any, node: Node) {
        events.push(["onUnreachableCodePathSegmentEnd", node.type]);
      },
      onCodePathSegmentLoop(fromSegment: any, toSegment: any, node: Node) {
        events.push(["onCodePathSegmentLoop", node.type]);
      },
    };
  },
};

// Purpose of this 2nd rule is to ensure that all arguments are passed to the CFG event handler functions
// when 2 event handler functions are merged into a single function.
// https://github.com/oxc-project/oxc/issues/18555
const rule2: Rule = {
  // @ts-expect-error - TODO: Make the types for CFG events work
  create(_context) {
    return {
      onCodePathSegmentLoop(_fromSegment: any, _toSegment: any, _node: Node) {
        // Do nothing
      },
    };
  },
};

const plugin: Plugin = {
  meta: {
    name: "cfg-plugin",
  },
  rules: {
    cfg: rule,
    noop: rule2,
  },
};

export default plugin;
