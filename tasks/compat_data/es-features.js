// https://github.com/babel/babel/blob/main/packages/babel-compat-data/scripts/data/plugin-features.js
// https://github.com/evanw/esbuild/blob/main/compat-table/src/index.ts

const f = (es) => (item) => {
  item.es = es;
  return item;
};

const es5 = [
  {
    name: 'ReservedWords',
    es: 'ES5',
    babel: 'transform-reserved-words',
    features: ['Miscellaneous / Unreserved words'],
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
    name: 'MemberExpressionLiterals',
    es: 'ES5',
    babel: 'transform-member-expression-literals',
    features: [
      'Object/array literal extensions / Reserved words as property names',
    ],
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
    name: 'Regenerator',
    babel: 'transform-regenerator',
    features: ['generators'],
  },
  {
    name: 'NewTarget',
    babel: 'transform-new-target',
    features: ['new.target', 'arrow functions / lexical "new.target" binding'],
  },
  {
    name: 'TypeofSymbol',
    babel: 'transform-typeof-symbol',
    features: ['Symbol / typeof support'],
  },
  {
    name: 'BlockScoping',
    babel: 'transform-block-scoping',
    features: ['const', 'let', 'generators'],
  },
  {
    name: 'Destructuring',
    babel: 'transform-destructuring',
    features: ['destructuring, assignment', 'destructuring, declarations'],
  },
  {
    name: 'Spread',
    babel: 'transform-spread',
    features: ['spread syntax for iterable objects', 'class', 'super'],
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
    name: 'UnicodeEscapes',
    babel: 'transform-unicode-escapes',
    features: ['Unicode code point escapes'],
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
    name: 'ForOf',
    babel: 'transform-for-of',
    features: ['for..of loops'],
  },
  {
    name: 'ComputedProperties',
    babel: 'transform-computed-properties',
    features: ['object literal extensions / computed properties'],
  },
  {
    name: 'DuplicateKeys',
    babel: 'transform-duplicate-keys',
    features: ['miscellaneous / duplicate property names in strict mode'],
  },
  {
    name: 'ShorthandProperties',
    babel: 'transform-shorthand-properties',
    features: ['object literal extensions / shorthand properties'],
  },
  {
    name: 'ObjectSuper',
    babel: 'transform-object-super',
    features: ['super'],
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
    name: 'BlockScopedFunctions',
    babel: 'transform-block-scoped-functions',
    features: ['block-level function declaration'],
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
    name: 'FunctionName',
    babel: 'transform-function-name',
    features: ['function "name" property'],
  },
  {
    name: 'Literals',
    babel: 'transform-literals',
    features: ['Unicode code point escapes'],
  },
  {
    name: 'TemplateLiterals',
    babel: 'transform-template-literals',
    features: ['template literals'],
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
    name: 'NamedCapturingGroupsRegex',
    babel: 'transform-named-capturing-groups-regex',
    features: ['RegExp named capture groups'],
  },
  {
    name: 'UnicodePropertyRegex',
    babel: 'transform-unicode-property-regex',
    features: ['RegExp Unicode Property Escapes / basic'],
  },
  {
    name: 'DotallRegex',
    babel: 'transform-dotall-regex',
    features: ['s (dotAll) flag for regular expressions'],
  },
  {
    name: 'LookbehindRegex',
    babel: null,
    features: ['RegExp Lookbehind Assertions'],
  },
  {
    name: 'ObjectRestSpread',
    babel: 'transform-object-rest-spread',
    features: ['object rest/spread properties'],
  },
  {
    name: 'AsyncGeneratorFunctions',
    babel: 'transform-async-generator-functions',
    features: ['Asynchronous Iterators'],
  },
  {
    name: 'OptionalCatchBinding',
    babel: 'transform-optional-catch-binding',
    features: ['optional catch binding'],
  },
].map(f('ES2018'));

const es2019 = [
  {
    name: 'JsonStrings',
    babel: 'transform-json-strings',
    features: ['JSON superset'],
  },
  {
    name: 'OptionalChaining',
    babel: 'transform-optional-chaining',
    features: ['optional chaining operator (?.)'],
  },
].map(f('ES2019'));

const es2020 = [
  {
    name: 'NullishCoalescingOperator',
    babel: 'transform-nullish-coalescing-operator',
    features: ['nullish coalescing operator (??)'],
  },
  {
    name: 'LogicalAssignmentOperators',
    babel: 'transform-logical-assignment-operators',
    features: ['Logical Assignment'],
  },
].map(f('ES2020'));

const es2021 = [
  {
    name: 'NumericSeparator',
    babel: 'transform-numeric-separator',
    features: ['numeric separators'],
  },
].map(f('ES2021'));

const es2022 = [
  {
    name: 'PrivateMethods',
    babel: 'transform-private-methods',
    features: ['private class methods'],
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
    name: 'PrivatePropertyInObject',
    babel: 'transform-private-property-in-object',
    features: ['Ergonomic brand checks for private fields'],
  },
  {
    name: 'ClassStaticBlock',
    babel: 'transform-class-static-block',
    features: ['Class static initialization blocks'],
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
    name: 'RegexpModifiers',
    babel: 'transform-regexp-modifiers',
    features: ['RegExp Pattern Modifiers'],
  },
  {
    name: 'DuplicateNamedCapturingGroupsRegex',
    babel: 'transform-duplicate-named-capturing-groups-regex',
    features: ['Duplicate named capturing groups'],
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
