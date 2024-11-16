// https://github.com/babel/babel/blob/v7.26.2/packages/babel-compat-data/scripts/data/plugin-features.js
// https://github.com/evanw/esbuild/blob/v0.24.0/compat-table/src/index.ts

const f = (es) => (item) => {
  item.es = es;
  return item;
};

const es5 = [
  {
    name: 'MemberExpressionLiterals',
    es: 'ES5',
    babel: 'transform-member-expression-literals',
    features: [
      'Object/array literal extensions / Reserved words as property names',
    ],
  },
  {
    name: 'PropertyLiterals',
    es: 'ES5',
    babel: 'transform-property-literals',
    features: [
      'Object/array literal extensions / Reserved words as property names',
    ],
  },
  {
    name: 'ReservedWords',
    es: 'ES5',
    babel: 'transform-reserved-words',
    features: ['Miscellaneous / Unreserved words'],
  },
].map(f('ES5'));

const es2015 = [
  {
    name: 'Parameters',
    babel: 'transform-parameters',
    features: [
      'default function parameters',
      'rest parameters',
      'destructuring, parameters / aliased defaults, arrow function',
      'destructuring, parameters / shorthand defaults, arrow function',
      'destructuring, parameters / duplicate identifier',
    ],
  },
  {
    name: 'TemplateLiterals',
    babel: 'transform-template-literals',
    features: ['template literals'],
  },
  {
    name: 'Literals',
    babel: 'transform-literals',
    features: ['Unicode code point escapes'],
  },
  {
    name: 'FunctionName',
    babel: 'transform-function-name',
    features: ['function "name" property'],
  },
  {
    name: 'ArrowFunctions',
    babel: 'transform-arrow-functions',
    features: [
      'arrow functions / 0 parameters',
      'arrow functions / 1 parameter, no brackets',
      'arrow functions / multiple parameters',
      'arrow functions / lexical "this" binding',
      'arrow functions / "this" unchanged by call or apply',
      "arrow functions / can't be bound, can be curried",
      'arrow functions / lexical "arguments" binding',
      'arrow functions / no line break between params and <code>=></code>',
      'arrow functions / correct precedence',
      'arrow functions / no "prototype" property',
    ],
  },
  {
    name: 'BlockScopedFunctions',
    babel: 'transform-block-scoped-functions',
    features: ['block-level function declaration'],
  },
  {
    name: 'Classes',
    babel: 'transform-classes',
    features: [
      'class',
      'super',
      'arrow functions / lexical "super" binding in constructors',
      'arrow functions / lexical "super" binding in methods',
    ],
  },
  {
    name: 'ObjectSuper',
    babel: 'transform-object-super',
    features: ['super'],
  },
  {
    name: 'ShorthandProperties',
    babel: 'transform-shorthand-properties',
    features: ['object literal extensions / shorthand properties'],
  },
  {
    name: 'DuplicateKeys',
    babel: 'transform-duplicate-keys',
    features: ['miscellaneous / duplicate property names in strict mode'],
  },
  {
    name: 'ComputedProperties',
    babel: 'transform-computed-properties',
    features: ['object literal extensions / computed properties'],
  },
  {
    name: 'ForOf',
    babel: 'transform-for-of',
    features: ['for..of loops'],
  },
  {
    name: 'StickyRegex',
    babel: 'transform-sticky-regex',
    features: [
      'RegExp "y" and "u" flags / "y" flag, lastIndex',
      'RegExp "y" and "u" flags / "y" flag',
    ],
  },
  {
    name: 'UnicodeEscapes',
    babel: 'transform-unicode-escapes',
    features: ['Unicode code point escapes'],
  },
  {
    name: 'UnicodeRegex',
    babel: 'transform-unicode-regex',
    features: [
      'RegExp "y" and "u" flags / "u" flag, case folding',
      'RegExp "y" and "u" flags / "u" flag, Unicode code point escapes',
      'RegExp "y" and "u" flags / "u" flag, non-BMP Unicode characters',
      'RegExp "y" and "u" flags / "u" flag',
    ],
  },
  {
    name: 'Spread',
    babel: 'transform-spread',
    features: ['spread syntax for iterable objects', 'class', 'super'],
  },
  {
    name: 'Destructuring',
    babel: 'transform-destructuring',
    features: ['destructuring, assignment', 'destructuring, declarations'],
  },
  {
    name: 'BlockScoping',
    babel: 'transform-block-scoping',
    features: ['const', 'let', 'generators'],
  },
  {
    name: 'TypeofSymbol',
    babel: 'transform-typeof-symbol',
    features: ['Symbol / typeof support'],
  },
  {
    name: 'NewTarget',
    babel: 'transform-new-target',
    features: ['new.target', 'arrow functions / lexical "new.target" binding'],
  },
  {
    name: 'Regenerator',
    babel: 'transform-regenerator',
    features: ['generators'],
  },
].map(f('ES2015'));

const es2016 = [
  {
    name: 'ExponentiationOperator',
    babel: 'transform-exponentiation-operator',
    features: ['exponentiation (**) operator'],
  },
].map(f('ES2016'));

const es2017 = [
  {
    name: 'AsyncToGenerator',
    babel: 'transform-async-to-generator',
    features: ['async functions'],
  },
].map(f('ES2017'));

const es2018 = [
  {
    name: 'AsyncGeneratorFunctions',
    babel: 'transform-async-generator-functions',
    features: ['Asynchronous Iterators'],
  },
  {
    name: 'ObjectRestSpread',
    babel: 'transform-object-rest-spread',
    features: ['object rest/spread properties'],
  },
  {
    name: 'DotallRegex',
    babel: 'transform-dotall-regex',
    features: ['s (dotAll) flag for regular expressions'],
  },
  {
    name: 'UnicodePropertyRegex',
    babel: 'transform-unicode-property-regex',
    features: ['RegExp Unicode Property Escapes / basic'],
  },
  {
    name: 'NamedCapturingGroupsRegex',
    babel: 'transform-named-capturing-groups-regex',
    features: ['RegExp named capture groups'],
  },
  {
    name: 'LookbehindRegex',
    babel: null,
    features: ['RegExp Lookbehind Assertions'],
  },
].map(f('ES2018'));

const es2019 = [
  {
    name: 'JsonStrings',
    babel: 'transform-json-strings',
    features: ['JSON superset'],
  },
  {
    name: 'OptionalCatchBinding',
    babel: 'transform-optional-catch-binding',
    features: ['optional catch binding'],
  },
].map(f('ES2019'));

const es2020 = [
  {
    name: 'NullishCoalescingOperator',
    babel: 'transform-nullish-coalescing-operator',
    features: ['nullish coalescing operator (??)'],
  },
  {
    name: 'OptionalChaining',
    babel: 'transform-optional-chaining',
    features: ['optional chaining operator (?.)'],
  },
  {
    name: 'BigInt',
    babel: null,
    features: ['BigInt / basic functionality'],
  },
].map(f('ES2020'));

const es2021 = [
  {
    name: 'NumericSeparator',
    babel: 'transform-numeric-separator',
    features: ['numeric separators'],
  },
  {
    name: 'LogicalAssignmentOperators',
    babel: 'transform-logical-assignment-operators',
    features: ['Logical Assignment'],
  },
].map(f('ES2021'));

const es2022 = [
  {
    name: 'ClassStaticBlock',
    babel: 'transform-class-static-block',
    features: ['Class static initialization blocks'],
  },
  {
    name: 'PrivatePropertyInObject',
    babel: 'transform-private-property-in-object',
    features: ['Ergonomic brand checks for private fields'],
  },
  {
    name: 'ClassProperties',
    babel: 'transform-class-properties',
    features: [
      'static class fields',
      'instance class fields / public instance class fields',
      'instance class fields / private instance class fields basic support',
      'instance class fields / computed instance class fields',
      'instance class fields / resolving identifier in parent scope',
    ],
  },
  {
    name: 'PrivateMethods',
    babel: 'transform-private-methods',
    features: ['private class methods'],
  },
  {
    name: 'MatchIndicesRegex',
    babel: null,
    features: [
      'RegExp Match Indices (`hasIndices` / `d` flag) / constructor supports it',
      // ignore "shows up in flags"
    ],
  },
].map(f('ES2022'));

const es2024 = [
  {
    name: 'UnicodeSetsRegex',
    babel: 'transform-unicode-sets-regex',
    features: [
      'RegExp `v` flag / set notations',
      'RegExp `v` flag / properties of Strings',
      'RegExp `v` flag / constructor supports it',
      'RegExp `v` flag / shows up in flags',
    ],
  },
].map(f('ES2024'));

const es2025 = [
  {
    name: 'DuplicateNamedCapturingGroupsRegex',
    babel: 'transform-duplicate-named-capturing-groups-regex',
    features: ['Duplicate named capturing groups'],
  },
  {
    name: 'RegexpModifiers',
    babel: 'transform-regexp-modifiers',
    features: ['RegExp Pattern Modifiers'],
  },
].map(f('ES2025'));

module.exports = [
  ...es5,
  ...es2015,
  ...es2016,
  ...es2017,
  ...es2018,
  ...es2019,
  ...es2020,
  ...es2021,
  ...es2022,
  ...es2024,
  ...es2025,
];
