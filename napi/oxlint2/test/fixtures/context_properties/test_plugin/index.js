import { dirname, sep } from 'node:path';

const SPAN = { start: 0, end: 0 };

const PARENT_DIR_PATH_LEN = dirname(import.meta.dirname).length + 1;

const relativePath = sep === '/'
  ? path => path.slice(PARENT_DIR_PATH_LEN)
  : path => path.slice(PARENT_DIR_PATH_LEN).replace(/\\/g, '/');

export default {
  meta: {
    name: "context-plugin",
  },
  rules: {
    "log-context": {
      create(context) {
        context.report({
          message: `id: ${context.id}`,
          node: SPAN,
        });

        context.report({
          message: `filename: ${relativePath(context.filename)}`,
          node: SPAN,
        });

        context.report({
          message: `physicalFilename: ${relativePath(context.physicalFilename)}`,
          node: SPAN,
        });

        context.report({
          message: `getFilename(): ${relativePath(context.getFilename())}`,
          node: SPAN,
        });

        context.report({
          message: `getPhysicalFilename(): ${relativePath(context.getPhysicalFilename())}`,
          node: SPAN,
        });

        return {};
      },
    },
  },
};
