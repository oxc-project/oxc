'use strict';

const fs = require('node:fs');
const pathJoin = require('node:path').join;
const tseslintTypes = require('@typescript-eslint/visitor-keys').visitorKeys;
const oxcTypes = require('./estree_field_orders.json');

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
