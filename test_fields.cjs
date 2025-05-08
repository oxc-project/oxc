'use strict';

const fs = require('node:fs');
const pathJoin = require('node:path').join;
const eslintTypes = require('@typescript-eslint/visitor-keys').visitorKeys;
const oxcTypes = require('./estree_field_orders.json');

const types = [...new Set([...Object.keys(eslintTypes), ...Object.keys(oxcTypes)])];
types.sort();

const VIRTUAL_FIELD_MISMATCHES = [
  'ArrayPattern',
  'AssignmentPattern',
  'ObjectPattern',
  'RestElement',
  'TSMappedType',
];

const TS_ESLINT_IS_WRONG = [
  // `local` should be before `exported` e.g. `export { local as exported }`
  'ExportSpecifier',
  // `options` should be before `qualifier` and `type_arguments`
  // e.g. `type T = import('foo', {assert:{}}).qualifier`
  // e.g. `type T = import('foo', {assert:{}})<T>`
  'TSImportType',
  // `objectType` should be before `indexType`
  'TSIndexedAccessType',
];

// Combine Oxc and TS-ESLint types
let combinedTypes = types.map((key) => {
  let oxc = oxcTypes[key];
  oxc = oxc ? oxc[0] : null;
  return { type: key, oxc, eslint: eslintTypes[key] || null };
});

// Filter out types which don't exist in both ASTs
combinedTypes = combinedTypes.filter(o => o.oxc && o.eslint);

// console.log(compared);

// Compare field orders
let errors = '';
for (const { type, oxc, eslint } of combinedTypes) {
  if (VIRTUAL_FIELD_MISMATCHES.includes(type) || TS_ESLINT_IS_WRONG.includes(type)) continue;

  const error = compare(oxc, eslint);
  if (error) errors += `${type}: ${error}\n\n`;
}
errors = errors.slice(0, -1);

fs.writeFileSync(pathJoin(__dirname, 'order_mismatches.txt'), errors);

function compare(oxc, eslint) {
  const missingFields = eslint.filter(key => !oxc.includes(key));
  if (missingFields.length > 0) {
    return `Missing fields: ${missingFields.join(', ')}`;
  }

  let lastIndex = -1;
  for (const key of eslint) {
    const index = oxc.indexOf(key);
    if (index < lastIndex) {
      return `Order mismatch: ${key}\nOxc: ${oxc.join(', ')}\nTS : ${eslint.join(', ')}`;
    }
    lastIndex = index;
  }
}
