export default {
  meta: {
    name: "basic-custom-plugin",
  },
  rules: {
    "no-debugger": {
      create(context) {
        return {
          DebuggerStatement(debuggerStatement) {
            context.report({
              message: "Unexpected Debugger Statement",
              node: debuggerStatement,
            });
          },
        };
      },
    },
    "no-debugger-2": {
      create(context) {
        return {
          DebuggerStatement(debuggerStatement) {
            context.report({
              message: "Unexpected Debugger Statement",
              node: debuggerStatement,
            });
          },
        };
      },
    },
    "no-ident-references-named-foo": {
      create(context) {
        return {
          IdentifierReference(identifierReference) {
            if (identifierReference.name == "foo") {
              context.report({
                message: "Unexpected Identifier Reference named foo",
                node: identifierReference,
              });
            }
          },
        };
      },
    },
  },
};
