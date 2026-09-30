import type { Plugin } from "#oxlint/plugins";

// Reports every `debugger` statement.
//
// This plugin exists so that the linter takes the JS plugin code path. That path consumes the AST
// of each `<script>` block as it lints it, which used to leave native rules that read the *other*
// `<script>` blocks of the same file (e.g. `vue/valid-define-emits`) looking at an emptied AST.
//
// It reports in both blocks, so the snapshot also records that JS plugins run on every block,
// not just the last one.
const plugin: Plugin = {
  meta: {
    name: "vue-scripts-plugin",
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
  },
};

export default plugin;
