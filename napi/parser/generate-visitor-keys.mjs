import { visitorKeys } from '@typescript-eslint/visitor-keys';
import { writeFileSync } from 'node:fs';

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

writeFileSync('./generated/visitor-keys.js', code);
