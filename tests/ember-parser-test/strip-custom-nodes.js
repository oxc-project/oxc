/**
 * Generic stripper for non-standard ESTree nodes
 *
 * Removes custom AST nodes that aren't part of the standard ESTree/TS-ESTree specification.
 * This allows the AST to be processed by tools that only understand standard JavaScript/TypeScript.
 *
 * Examples of custom nodes:
 * - Glimmer nodes (GlimmerTemplate, GlimmerElementNode, etc.) from ember-eslint-parser
 * - Vue SFC nodes from vue-eslint-parser
 * - Svelte nodes from svelte-eslint-parser
 * - Any other framework-specific AST extensions
 */

// Standard ESTree node types (ES2022 + Stage 4 proposals)
const STANDARD_ESTREE_TYPES = new Set([
  // Program
  'Program',

  // Statements
  'ExpressionStatement',
  'BlockStatement',
  'EmptyStatement',
  'DebuggerStatement',
  'WithStatement',
  'ReturnStatement',
  'LabeledStatement',
  'BreakStatement',
  'ContinueStatement',
  'IfStatement',
  'SwitchStatement',
  'SwitchCase',
  'ThrowStatement',
  'TryStatement',
  'CatchClause',
  'WhileStatement',
  'DoWhileStatement',
  'ForStatement',
  'ForInStatement',
  'ForOfStatement',

  // Declarations
  'FunctionDeclaration',
  'VariableDeclaration',
  'VariableDeclarator',
  'ClassDeclaration',
  'ClassBody',
  'MethodDefinition',
  'PropertyDefinition',
  'StaticBlock',

  // Expressions
  'ThisExpression',
  'ArrayExpression',
  'ObjectExpression',
  'Property',
  'FunctionExpression',
  'UnaryExpression',
  'UpdateExpression',
  'BinaryExpression',
  'AssignmentExpression',
  'LogicalExpression',
  'MemberExpression',
  'ConditionalExpression',
  'CallExpression',
  'NewExpression',
  'SequenceExpression',
  'ArrowFunctionExpression',
  'YieldExpression',
  'AwaitExpression',
  'TemplateLiteral',
  'TemplateElement',
  'TaggedTemplateExpression',
  'ClassExpression',
  'MetaProperty',
  'Super',
  'SpreadElement',
  'ChainExpression',
  'ImportExpression',
  'ParenthesizedExpression',

  // Patterns
  'ObjectPattern',
  'ArrayPattern',
  'RestElement',
  'AssignmentPattern',

  // Literals
  'Identifier',
  'Literal',
  'RegExpLiteral',
  'BigIntLiteral',

  // Modules
  'ImportDeclaration',
  'ImportSpecifier',
  'ImportDefaultSpecifier',
  'ImportNamespaceSpecifier',
  'ExportNamedDeclaration',
  'ExportSpecifier',
  'ExportDefaultDeclaration',
  'ExportAllDeclaration',
  'ImportAttribute',

  // Private identifiers
  'PrivateIdentifier',

  // Decorators (Stage 3)
  'Decorator',
]);

// TypeScript-specific node types (TS-ESTree)
const TYPESCRIPT_ESTREE_TYPES = new Set([
  // TypeScript types
  'TSAnyKeyword',
  'TSBigIntKeyword',
  'TSBooleanKeyword',
  'TSIntrinsicKeyword',
  'TSNeverKeyword',
  'TSNullKeyword',
  'TSNumberKeyword',
  'TSObjectKeyword',
  'TSStringKeyword',
  'TSSymbolKeyword',
  'TSUndefinedKeyword',
  'TSUnknownKeyword',
  'TSVoidKeyword',
  'TSThisType',
  'TSArrayType',
  'TSConditionalType',
  'TSConstructorType',
  'TSFunctionType',
  'TSImportType',
  'TSIndexedAccessType',
  'TSInferType',
  'TSIntersectionType',
  'TSLiteralType',
  'TSMappedType',
  'TSNamedTupleMember',
  'TSOptionalType',
  'TSParenthesizedType',
  'TSRestType',
  'TSTemplateLiteralType',
  'TSTupleType',
  'TSTypeAnnotation',
  'TSTypeLiteral',
  'TSTypeOperator',
  'TSTypeParameter',
  'TSTypeParameterDeclaration',
  'TSTypeParameterInstantiation',
  'TSTypePredicate',
  'TSTypeQuery',
  'TSTypeReference',
  'TSUnionType',
  'TSQualifiedName',

  // TypeScript declarations
  'TSInterfaceDeclaration',
  'TSInterfaceBody',
  'TSInterfaceHeritage',
  'TSModuleDeclaration',
  'TSModuleBlock',
  'TSEnumDeclaration',
  'TSEnumMember',
  'TSTypeAliasDeclaration',
  'TSNamespaceExportDeclaration',
  'TSExportAssignment',
  'TSImportEqualsDeclaration',
  'TSExternalModuleReference',

  // TypeScript expressions
  'TSAsExpression',
  'TSTypeAssertion',
  'TSNonNullExpression',
  'TSSatisfiesExpression',
  'TSInstantiationExpression',

  // TypeScript signatures
  'TSMethodSignature',
  'TSPropertySignature',
  'TSIndexSignature',
  'TSCallSignatureDeclaration',
  'TSConstructSignatureDeclaration',

  // TypeScript modifiers
  'TSAbstractKeyword',
  'TSAbstractMethodDefinition',
  'TSAbstractPropertyDefinition',

  // JSDoc types (used by TypeScript)
  'JSDocAllType',
  'JSDocUnknownType',
  'JSDocNullableType',
  'JSDocNonNullableType',
  'JSDocOptionalType',
  'JSDocFunctionType',
  'JSDocVariadicType',
  'JSDocNamepathType',
]);

// Combine standard and TypeScript types
const KNOWN_ESTREE_TYPES = new Set([
  ...STANDARD_ESTREE_TYPES,
  ...TYPESCRIPT_ESTREE_TYPES,
]);

/**
 * Check if a node type is a known ESTree type
 */
function isKnownESTreeType(type) {
  return KNOWN_ESTREE_TYPES.has(type);
}

/**
 * Check if a node is a custom (non-ESTree) node
 */
function isCustomNode(node) {
  if (!node || typeof node !== 'object' || !node.type) {
    return false;
  }
  return !isKnownESTreeType(node.type);
}

/**
 * Collect all custom node types in an AST
 */
function collectCustomNodeTypes(node, customTypes = new Set()) {
  if (!node || typeof node !== 'object') {
    return customTypes;
  }

  if (node.type && isCustomNode(node)) {
    customTypes.add(node.type);
  }

  // Recurse into child nodes
  if (Array.isArray(node)) {
    node.forEach(child => collectCustomNodeTypes(child, customTypes));
  } else {
    for (const key in node) {
      if (key !== 'parent' && key !== 'loc' && key !== 'range') {
        collectCustomNodeTypes(node[key], customTypes);
      }
    }
  }

  return customTypes;
}

/**
 * Strip custom nodes from an AST
 *
 * Strategy:
 * 1. If custom node is in statement position: replace with empty statement or comment
 * 2. If custom node is in expression position: replace with null literal
 * 3. If custom node is in array: filter it out
 * 4. Preserve location information for debugging
 */
function stripCustomNodes(ast, options = {}) {
  const {
    preserveLocations = true,
    replacementComment = 'Custom node removed for standard ESTree processing',
    verbose = false,
  } = options;

  const stats = {
    nodesStripped: 0,
    customTypes: new Set(),
  };

  function createReplacement(node, context) {
    stats.nodesStripped++;
    stats.customTypes.add(node.type);

    const base = preserveLocations ? {
      range: node.range,
      loc: node.loc,
    } : {};

    if (verbose) {
      console.log(`Stripping ${node.type} at ${context}`);
    }

    // For statement positions: use ExpressionStatement with comment
    if (context === 'statement' || context === 'body') {
      return {
        type: 'ExpressionStatement',
        expression: {
          type: 'Literal',
          value: `[${node.type} removed]`,
          raw: `"[${node.type} removed]"`,
          ...base,
        },
        directive: replacementComment,
        ...base,
      };
    }

    // For expression positions: use null literal
    if (context === 'expression') {
      return {
        type: 'Literal',
        value: null,
        raw: 'null',
        ...base,
      };
    }

    // Default: null literal
    return {
      type: 'Literal',
      value: null,
      raw: 'null',
      ...base,
    };
  }

  function traverse(node, parent = null, key = null, context = 'unknown') {
    if (!node || typeof node !== 'object') {
      return node;
    }

    // Handle arrays
    if (Array.isArray(node)) {
      return node
        .map((item, index) => traverse(item, node, index, context))
        .filter(item => item !== null); // Remove nulls from arrays
    }

    // Check if this is a custom node
    if (node.type && isCustomNode(node)) {
      // Determine context for replacement
      const inferredContext =
        key === 'body' ? 'body' :
        key === 'expression' ? 'expression' :
        key === 'init' || key === 'test' || key === 'update' ? 'expression' :
        context;

      return createReplacement(node, inferredContext);
    }

    // Traverse known ESTree nodes
    const cloned = { ...node };

    for (const key in cloned) {
      if (key === 'parent') continue; // Skip parent references

      const value = cloned[key];

      if (value && typeof value === 'object') {
        // Determine context for child nodes
        const childContext =
          key === 'body' || key === 'consequent' || key === 'alternate' ? 'body' :
          key === 'expression' || key === 'argument' || key === 'test' ? 'expression' :
          'unknown';

        cloned[key] = traverse(value, cloned, key, childContext);
      }
    }

    return cloned;
  }

  const result = traverse(ast);

  return {
    ast: result,
    stats,
  };
}

/**
 * Validate that an AST contains only known ESTree nodes
 */
function validateESTreeAST(ast) {
  const customTypes = collectCustomNodeTypes(ast);

  if (customTypes.size > 0) {
    return {
      valid: false,
      customTypes: Array.from(customTypes),
    };
  }

  return {
    valid: true,
    customTypes: [],
  };
}

export {
  stripCustomNodes,
  collectCustomNodeTypes,
  isCustomNode,
  isKnownESTreeType,
  validateESTreeAST,
  KNOWN_ESTREE_TYPES,
  STANDARD_ESTREE_TYPES,
  TYPESCRIPT_ESTREE_TYPES,
};
