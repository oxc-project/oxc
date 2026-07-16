import { definePlugin, defineRule } from "#oxlint/plugins";

/**
 * Example JS rule plugin that consumes the Vue-native AST from the language plugin.
 *
 * From RFC #21936 — rules listen for framework nodes such as `VElement`.
 */
export default definePlugin({
  meta: { name: "vue-poc-plugin" },
  rules: {
    "report-div": defineRule({
      create(context) {
        return {
          VElement(node) {
            // Language ASTs are not in the generated ESTree visitor typings yet.
            const element = node as { tag?: string };
            if (element.tag === "div") {
              context.report({
                message: "div from vue ast",
                node,
              });
            }
          },
        };
      },
    }),
  },
});
