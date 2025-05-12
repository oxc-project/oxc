'use strict';

const fs = require('node:fs');
const pathJoin = require('node:path').join;
let tseslintTypes = require('@typescript-eslint/visitor-keys').visitorKeys;
const oxcTypes = require('./estree_field_orders.json');

// Correct mistakes in TS-ESLint visitation order
tseslintTypes = {...tseslintTypes};
function fixTseslintKeys(type, keys) {
  const existingKeys = [...tseslintTypes[type]];
  existingKeys.sort();
  const newKeys = [...keys];
  newKeys.sort();
  if (newKeys.length !== existingKeys.length || newKeys.some((key, i) => key !== existingKeys[i])) {
    throw new Error(`New keys for ${type} differ from old in more than order`);
  }

  tseslintTypes[type] = keys;
}

// `local` should be before `exported`
// e.g. `export { local as exported }`
fixTseslintKeys('ExportSpecifier', ['local', 'exported']);
// `options` should be before `qualifier` and `type_arguments`
// e.g. `type T = import('foo', {assert:{}}).qualifier`
// e.g. `type T = import('foo', {assert:{}})<T>`
fixTseslintKeys('TSImportType', ['argument', 'options', 'qualifier', 'typeArguments']);
// `objectType` should be before `indexType`
fixTseslintKeys('TSIndexedAccessType', ['objectType', 'indexType']);
// `key` should be before `typeParameters`
// e.g. `interface Foo { bar<T>(a: number): string; }`
fixTseslintKeys('TSMethodSignature', ['key', 'typeParameters', 'params', 'returnType']);
// `key` should be before `typeParameters`
// e.g. `interface Foo { bar: number }`
fixTseslintKeys('TSPropertySignature', ['key', 'typeAnnotation']);
// `parameterName` should be before `typeAnnotation`
// e.g. `function isString(x: unknown): x is string {}`
fixTseslintKeys('TSTypePredicate', ['parameterName', 'typeAnnotation']);

// Combine Oxc and TS-ESLint types
const types = [...new Set([...Object.keys(tseslintTypes), ...Object.keys(oxcTypes)])];
types.sort();

let combinedTypes = types.map((key) => ({
  type: key,
  oxc: oxcTypes[key] || null,
  tseslint: tseslintTypes[key] || null
}));

// Filter out types which don't exist in both TS-ESLint and Oxc ASTs
combinedTypes = combinedTypes.filter(o => o.oxc && o.tseslint);

// Compare field orders
let errors = '';
for (const { type, oxc, tseslint } of combinedTypes) {
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
