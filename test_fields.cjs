'use strict';

const fs = require('node:fs');
const pathJoin = require('node:path').join;
const tseslintTypes = require('@typescript-eslint/visitor-keys').visitorKeys;
const oxcTypes = require('./estree_field_orders.json');

const types = [...new Set([...Object.keys(tseslintTypes), ...Object.keys(oxcTypes)])];
types.sort();

const VIRTUAL_FIELD_MISMATCHES = [
  'ArrayPattern',
  'AssignmentPattern',
  'ObjectPattern',
  'RestElement',
  'TSMappedType',
];

const TS_ESLINT_IS_WRONG = [
  // `local` should be before `exported`
  // e.g. `export { local as exported }`
  'ExportSpecifier',
  // `options` should be before `qualifier` and `type_arguments`
  // e.g. `type T = import('foo', {assert:{}}).qualifier`
  // e.g. `type T = import('foo', {assert:{}})<T>`
  'TSImportType',
  // `objectType` should be before `indexType`
  'TSIndexedAccessType',
  // `key` should be before `typeParameters`
  // e.g. `interface Foo { bar<T>(a: number): string; }`
  'TSMethodSignature',
  // `key` should be before `typeParameters`
  // e.g. `interface Foo { bar: number }`
  'TSPropertySignature',
  // `parameterName` should be before `typeAnnotation`
  // e.g. `function isString(x: unknown): x is string {}`
  'TSTypePredicate',
];

// Combine Oxc and TS-ESLint types
let combinedTypes = types.map((key) => {
  let oxc = oxcTypes[key];
  oxc = oxc ? oxc[0] : null;
  return { type: key, oxc, tseslint: tseslintTypes[key] || null };
});

// Filter out types which don't exist in both TS-ESLint and Oxc ASTs
combinedTypes = combinedTypes.filter(o => o.oxc && o.tseslint);

// console.log(compared);

// Compare field orders
let errors = '';
for (const { type, oxc, tseslint } of combinedTypes) {
  if (VIRTUAL_FIELD_MISMATCHES.includes(type) || TS_ESLINT_IS_WRONG.includes(type)) continue;

  const error = compare(oxc, tseslint);
  if (error) errors += `${type}: ${error}\n\n`;
}
errors = errors.slice(0, -1);

fs.writeFileSync(pathJoin(__dirname, 'order_mismatches.txt'), errors);

function compare(oxc, tseslint) {
  const missingFields = tseslint.filter(key => !oxc.includes(key));
  if (missingFields.length > 0) {
    return `Missing fields: ${missingFields.join(', ')}`;
  }

  let lastIndex = -1;
  for (const key of tseslint) {
    const index = oxc.indexOf(key);
    if (index < lastIndex) {
      return `Order mismatch: ${key}\nOxc: ${oxc.join(', ')}\nTS : ${tseslint.join(', ')}`;
    }
    lastIndex = index;
  }
}
