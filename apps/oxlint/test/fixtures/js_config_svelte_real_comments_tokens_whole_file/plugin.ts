import type { Plugin } from "#oxlint/plugins";

type TokenOrComment = {
  type?: string;
  value?: string;
  start?: number;
  end?: number;
  range?: [number, number];
};

const COMMENT_TEXT = "<!--A-->";

const plugin: Plugin = {
  meta: {
    name: "real-svelte-comments",
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
            const firstToken = context.sourceCode.getFirstToken(node) as TokenOrComment | null;
            const lastToken = context.sourceCode.getLastToken(node) as TokenOrComment | null;
            const astTokens = context.sourceCode.ast.tokens as TokenOrComment[];
            const nodeRange = (node as { range: [number, number] }).range;

            context.report({
              message: [
                `commentSpan=${
                  allComments.length === 1 &&
                  allComments[0].start === 0 &&
                  allComments[0].end === COMMENT_TEXT.length
                }`,
                `commentValue=${allComments[0]?.value?.trim() === "A"}`,
                `beforeHasComment=${commentsBefore.length === 1 && commentsBefore[0] === allComments[0]}`,
                `hasTokens=${tokens.length > 0}`,
                `mergedStartsWithComment=${merged[0] === allComments[0]}`,
                `firstInsideNode=${
                  firstToken?.range != null &&
                  nodeRange[0] <= firstToken.range[0] &&
                  firstToken.range[1] <= nodeRange[1]
                }`,
                `lastInsideNode=${
                  lastToken?.range != null &&
                  nodeRange[0] <= lastToken.range[0] &&
                  lastToken.range[1] <= nodeRange[1]
                }`,
                `astComments=${context.sourceCode.ast.comments === allComments}`,
                `astTokens=${Array.isArray(astTokens) && astTokens.length > 0 && astTokens.some((token) => token === firstToken)}`,
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
