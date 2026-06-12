import path from "node:path";

function isParentFolder(relativeFilePath, context, rootDir) {
  const absoluteRootPath = path.join(context.getCwd(), rootDir);
  const absoluteFilePath = path.join(path.dirname(context.getFilename()), relativeFilePath);

  return (
    relativeFilePath.startsWith("../") &&
    (rootDir === "" ||
      (absoluteFilePath.startsWith(absoluteRootPath) &&
        context.getFilename().startsWith(absoluteRootPath)))
  );
}

function getRelativePathDepth(importPath) {
  let depth = 0;
  while (importPath.startsWith("../")) {
    depth += 1;
    importPath = importPath.slice(3);
  }
  return depth;
}

function getAbsolutePath(relativeFilePath, context, rootDir, prefix) {
  return [
    prefix,
    ...path
      .relative(
        path.join(context.getCwd(), rootDir),
        path.join(path.dirname(context.getFilename()), relativeFilePath),
      )
      .split(path.sep),
  ]
    .filter(String)
    .join("/");
}

const message = "import statements should have an absolute path";

const plugin = {
  meta: {
    name: "nested-imports",
  },
  rules: {
    "no-relative-import-paths": {
      meta: {
        fixable: "code",
        schema: [
          {
            type: "object",
            properties: {
              allowedDepth: { type: "number" },
              rootDir: { type: "string" },
              prefix: { type: "string" },
            },
            additionalProperties: false,
          },
        ],
      },
      create(context) {
        const { allowedDepth, rootDir, prefix } = {
          allowedDepth: context.options[0]?.allowedDepth,
          rootDir: context.options[0]?.rootDir || "",
          prefix: context.options[0]?.prefix || "",
        };

        return {
          ImportDeclaration(node) {
            const importPath = node.source.value;
            if (!isParentFolder(importPath, context, rootDir)) return;
            if (
              typeof allowedDepth !== "undefined" &&
              getRelativePathDepth(importPath) <= allowedDepth
            ) {
              return;
            }

            context.report({
              node,
              message,
              fix: (fixer) =>
                fixer.replaceTextRange(
                  [node.source.range[0] + 1, node.source.range[1] - 1],
                  getAbsolutePath(importPath, context, rootDir, prefix),
                ),
            });
          },
        };
      },
    },
  },
};

export default plugin;
