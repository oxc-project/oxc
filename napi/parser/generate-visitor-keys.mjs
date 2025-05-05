import { visitorKeys as visitorKeysOriginal } from '@typescript-eslint/visitor-keys';
import { writeFileSync } from 'node:fs';
import { join as pathJoin } from 'node:path';

const PATH = pathJoin(import.meta.dirname, 'generated/visitor-keys.js');

// Add keys for `ParenthesizedExpression` and `TSParenthesizedType`, which TS-ESLint doesn't have
const visitorKeys = {
  ...visitorKeysOriginal,
  ParenthesizedExpression: ['expression'],
  TSParenthesizedType: ['typeAnnotation'],
};

const keys = Object.entries(visitorKeys)
  .filter(([, v]) => v?.length)
  .map(([k, v]) => `  ${k}: [${v.map((v) => `'${v}'`).join(', ')}],`)
  .join('\n');

const code = `// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit \`napi/parser/generate-visitor-keys.mjs\`.

module.exports = {
${keys}
};
`;

writeFileSync(PATH, code);
