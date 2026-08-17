// Slow-JS corpus rule: a naming-convention check that walks the raw-transfer AST.
//
// Per `Identifier` it reads the node's own lazily-deserialized fields, walks its ancestor
// chain to work out the enclosing scope path, and slices the source text -- the same shape of
// work a real `camelcase`/`id-naming` rule does. There is no sleep and no busy loop; every
// millisecond is AST access.
//
// `PASSES` scales the ancestor/text work per identifier so the run is dominated by JS plugin
// time rather than by Rust parse time. Set via OXLINT_BENCH_PASSES.

// 4 is what it takes for the corpus to stop being parse-bound: parse-help
// (`lint_ms(16)/lint_ms(1)` on one isolate) is 0.87 at 1 pass and 0.835 at 4.
const PASSES = Number(process.env.OXLINT_BENCH_PASSES ?? 4);

const SCOPE_TYPES = new Set([
  "FunctionDeclaration",
  "FunctionExpression",
  "ArrowFunctionExpression",
  "ClassDeclaration",
  "ClassExpression",
  "MethodDefinition",
  "Program",
]);

const CAMEL_CASE = /^(?:_*[a-z][a-zA-Z0-9]*|_*[A-Z][A-Z0-9_]*|_*[A-Z][a-zA-Z0-9]*)$/;

const plugin = {
  meta: { name: "bench" },
  rules: {
    "slow-walk": {
      create(context) {
        const { sourceCode } = context;
        return {
          Identifier(node) {
            const { name } = node;
            let flagged = false;

            for (let pass = 0; pass < PASSES; pass++) {
              // Ancestor chain: reads `parent` up the tree, deserializing each node.
              const ancestors = sourceCode.getAncestors(node);
              let scopePath = "";
              let depth = 0;
              for (let i = ancestors.length - 1; i >= 0; i--) {
                const ancestor = ancestors[i];
                const { type } = ancestor;
                if (!SCOPE_TYPES.has(type)) continue;
                depth++;
                const { id, key } = ancestor;
                scopePath = `${type}:${id ? id.name : key ? key.name : "?"}/${scopePath}`;
              }

              // Source text of the identifier plus a little context, as a rule producing a
              // suggestion would need.
              const text = sourceCode.getText(node, 1, 1);
              const [start, end] = node.range;

              if (
                depth > 0 &&
                end > start &&
                text.length > 0 &&
                scopePath.length > 0 &&
                !CAMEL_CASE.test(name)
              ) {
                flagged = true;
              }
            }

            // Deliberately near-zero diagnostics: the benchmark measures walking, not printing.
            if (flagged && name === "__oxlint_bench_never_matches__") {
              context.report({ message: `bad name \`${name}\``, node });
            }
          },
        };
      },
    },
  },
};

export default plugin;
