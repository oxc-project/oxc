import type { Plugin } from "#oxlint/plugins";

type TokenOrComment = {
  type: string;
  value: string;
};

function format(entries: TokenOrComment[]): string {
  return entries.map(({ type, value }) => `${type}:${value}`).join(", ");
}

const plugin: Plugin = {
  meta: {
    name: "whole-file-svelte-source",
  },
  rules: {
    "tokens-and-comments": {
      create(context) {
        return {
          SvelteElement(node) {
            const allComments = context.sourceCode.getAllComments() as TokenOrComment[];
            const commentsBefore = context.sourceCode.getCommentsBefore(node) as TokenOrComment[];
            const tokens = context.sourceCode.getTokens(node) as TokenOrComment[];
            const merged = context.sourceCode.tokensAndComments as TokenOrComment[];
            const firstToken = context.sourceCode.getFirstToken(node);
            const lastToken = context.sourceCode.getLastToken(node);
            const astTokens = context.sourceCode.ast.tokens as TokenOrComment[];

            context.report({
              message: [
                `allComments=${format(allComments)}`,
                `before=${format(commentsBefore)}`,
                `tokens=${format(tokens)}`,
                `merged=${format(merged)}`,
                `first=${firstToken?.value ?? "<missing>"}`,
                `last=${lastToken?.value ?? "<missing>"}`,
                `astComments=${context.sourceCode.ast.comments === allComments}`,
                `astTokens=${Array.isArray(astTokens) && astTokens[0] === firstToken}`,
              ].join("; "),
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
