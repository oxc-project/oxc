import { visitorKeys } from '@typescript-eslint/visitor-keys';
import { writeFileSync } from 'node:fs';

writeFileSync(
  './generated/visitor-keys.js',
  `// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit \`napi/parser/generate-visitor-keys.mjs\`.

module.exports = {${
    Object.entries(visitorKeys)
      .filter(([, v]) => v?.length)
      .map(([k, v]) => `\n  ${k}: [${v.map((v) => `"${v}"`).join(', ')}],`)
      .join('')
  }
};
`,
);
