/**
 * Generic stripper for non-standard ESTree nodes
 *
 * Removes custom AST nodes that aren't part of the standard ESTree/TS-ESTree specification.
 * This allows custom parser output to be processed by oxc's standard tooling.
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
const KNOWN_ESTREE_TYPES = new Set([...STANDARD_ESTREE_TYPES, ...TYPESCRIPT_ESTREE_TYPES]);

/**
 * Check if a node type is a known ESTree type
 */
function isKnownESTreeType(type: string): boolean {
  return KNOWN_ESTREE_TYPES.has(type);
}

/**
 * Check if a node is a custom (non-ESTree) node
 */
function isCustomNode(node: any): boolean {
  if (!node || typeof node !== 'object' || !node.type) {
    return false;
  }
  return !isKnownESTreeType(node.type);
}

interface StripOptions {
  preserveLocations?: boolean;
  replacementComment?: string;
}

interface StripResult {
  ast: any;
  stats: {
    nodesStripped: number;
    customTypes: Set<string>;
  };
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
export function stripCustomNodes(ast: any, options: StripOptions = {}): StripResult {
  const { preserveLocations = true, replacementComment = 'Custom node removed for standard ESTree processing' } =
    options;

  const stats = {
    nodesStripped: 0,
    customTypes: new Set<string>(),
  };

  function createReplacement(node: any, context: string): any {
    stats.nodesStripped++;
    stats.customTypes.add(node.type);

    const base = preserveLocations
      ? {
          range: node.range,
          loc: node.loc,
        }
      : {};

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

  /**
   * Convert a custom path-like expression to a standard ESTree MemberExpression.
   * This is a generic heuristic that works for any custom parser by looking for
   * common patterns that represent JavaScript property access paths.
   * 
   * Works generically by:
   * - Looking for "path" or "original" properties (common in template parsers)
   * - Handling arrays, strings, or nested objects
   * - Converting to standard ESTree MemberExpression for semantic analysis
   * 
   * If a custom parser uses a different structure, standard ESTree expressions
   * embedded within custom nodes will still be found and preserved.
   */
  function convertPathLikeToMemberExpression(node: any, base: any): any {
    // Generic heuristic: look for common patterns in custom nodes that represent JavaScript paths
    // This is not parser-specific - we're just looking for structural patterns:
    // - A "path" property that's an array of strings (e.g., ["this", "count"])
    // - A "path" property that's a string (e.g., "this.count")
    // - An "original" property that might contain the path
    // - Nested path objects with "original" or "name" properties
    let pathParts: string[] | null = null;

    if (node.path) {
      if (Array.isArray(node.path)) {
        pathParts = node.path.map((p: any) => {
          if (typeof p === 'string') return p;
          if (p && typeof p === 'object' && p.original) return p.original;
          if (p && typeof p === 'object' && p.name) return p.name;
          return String(p);
        });
      } else if (typeof node.path === 'string') {
        pathParts = node.path.split('.');
      } else if (node.path.original) {
        if (Array.isArray(node.path.original)) {
          pathParts = node.path.original;
        } else if (typeof node.path.original === 'string') {
          pathParts = node.path.original.split('.');
        }
      }
    } else if (node.original) {
      if (Array.isArray(node.original)) {
        pathParts = node.original;
      } else if (typeof node.original === 'string') {
        pathParts = node.original.split('.');
      }
    }

    if (!pathParts || pathParts.length === 0) {
      return null;
    }

    // Build a MemberExpression chain from the path parts
    // e.g., ["this", "count"] becomes MemberExpression(this, Identifier("count"))
    let current: any = null;
    for (let i = 0; i < pathParts.length; i++) {
      const part = pathParts[i];
      if (i === 0) {
        // First part: could be "this", a variable name, etc.
        if (part === 'this') {
          current = {
            type: 'ThisExpression',
            ...base,
          };
        } else {
          current = {
            type: 'Identifier',
            name: part,
            ...base,
          };
        }
      } else {
        // Subsequent parts: property access
        current = {
          type: 'MemberExpression',
          object: current,
          property: {
            type: 'Identifier',
            name: part,
            ...base,
          },
          computed: false,
          optional: false,
          ...base,
        };
      }
    }

    return current;
  }

  /**
   * Extract JavaScript expressions from a custom node for semantic analysis.
   * This allows rules like no-unused-vars to detect variable usage in templates.
   * Works generically for any custom parser by:
   * 1. Finding embedded standard ESTree expressions
   * 2. Converting custom path-like nodes to MemberExpressions
   */
  function extractJSExpressionsFromCustomNode(customNode: any, visited = new WeakSet(), expressionSet = new Map()): any[] {
    if (!customNode || typeof customNode !== 'object' || visited.has(customNode)) {
      return [];
    }
    visited.add(customNode);

    const expressions: any[] = [];
    const base = preserveLocations
      ? {
          range: customNode.range,
          loc: customNode.loc,
        }
      : {};

    // If this is a standard ESTree expression node, collect it
    const standardExpressionTypes = [
      'Identifier',
      'MemberExpression',
      'CallExpression',
      'ThisExpression',
      'Super',
      'Literal',
      'ArrayExpression',
      'ObjectExpression',
      'FunctionExpression',
      'ArrowFunctionExpression',
      'ClassExpression',
      'TaggedTemplateExpression',
      'TemplateLiteral',
      'BinaryExpression',
      'LogicalExpression',
      'UnaryExpression',
      'UpdateExpression',
      'AssignmentExpression',
      'ConditionalExpression',
      'NewExpression',
      'SequenceExpression',
      'YieldExpression',
      'AwaitExpression',
      'ImportExpression',
      'ChainExpression',
    ];

    // Check if this node itself is a standard expression
    if (customNode.type && standardExpressionTypes.includes(customNode.type)) {
      // Create a key for deduplication based on the expression structure
      const key = getExpressionKey(customNode);
      if (!expressionSet.has(key)) {
        expressionSet.set(key, customNode);
        expressions.push(customNode);
      }
    }

    // Try to convert custom path-like nodes to MemberExpression
    // Look for nodes that might represent JavaScript property access
    // Common patterns: nodes with "path" property, nodes ending in "PathExpression", etc.
    if (customNode.type && !standardExpressionTypes.includes(customNode.type)) {
      const converted = convertPathLikeToMemberExpression(customNode, base);
      if (converted) {
        const key = getExpressionKey(converted);
        if (!expressionSet.has(key)) {
          expressionSet.set(key, converted);
          expressions.push(converted);
        }
      }
    }

    // Recursively search children for standard expressions and path-like nodes
    // But skip if we've already processed this as a path-like node to avoid duplicates
    if (!customNode.type || !standardExpressionTypes.includes(customNode.type)) {
      for (const key in customNode) {
        if (key === 'parent' || key === 'loc' || key === 'range' || key === 'type') continue;
        const value = customNode[key];
        if (Array.isArray(value)) {
          value.forEach(item => {
            expressions.push(...extractJSExpressionsFromCustomNode(item, visited, expressionSet));
          });
        } else if (value && typeof value === 'object') {
          expressions.push(...extractJSExpressionsFromCustomNode(value, visited, expressionSet));
        }
      }
    }

    return expressions;
  }

  /**
   * Create a unique key for an expression to deduplicate.
   * Uses the expression structure to identify duplicates.
   */
  function getExpressionKey(expr: any): string {
    if (!expr || typeof expr !== 'object') return String(expr);
    
    if (expr.type === 'Identifier') {
      return `Identifier:${expr.name}`;
    }
    if (expr.type === 'MemberExpression') {
      const objKey = getExpressionKey(expr.object);
      const propKey = getExpressionKey(expr.property);
      return `MemberExpression:${objKey}.${propKey}`;
    }
    if (expr.type === 'ThisExpression') {
      return 'ThisExpression';
    }
    if (expr.type === 'CallExpression') {
      const calleeKey = getExpressionKey(expr.callee);
      return `CallExpression:${calleeKey}`;
    }
    
    // For other types, use type + a hash of the structure
    return `${expr.type}:${JSON.stringify(expr).slice(0, 100)}`;
  }

  /**
   * Create a synthetic method that references extracted JavaScript expressions.
   * This allows semantic analysis to see variable usage from custom nodes (e.g., templates).
   * 
   * We use a pattern that won't trigger "unused expression" warnings:
   * - Wrap expressions in void() to mark them as intentionally evaluated
   * - This is a generic pattern that works for any custom parser
   */
  function createSyntheticMethodForExpressions(expressions: any[], originalNode: any): any {
    if (expressions.length === 0) {
      return null; // No expressions to preserve
    }

    const base = preserveLocations
      ? {
          range: originalNode.range,
          loc: originalNode.loc,
        }
      : {};

    // Create statements that use the expressions in a way that won't trigger "unused" warnings
    // We wrap each expression in void() which is a common pattern for intentionally
    // evaluating expressions for side effects (in this case, for semantic analysis)
    const statements = expressions.map(expr => ({
      type: 'ExpressionStatement',
      expression: {
        type: 'UnaryExpression',
        operator: 'void',
        prefix: true,
        argument: expr,
        ...base,
      },
      ...base,
    }));

    return {
      type: 'MethodDefinition',
      key: {
        type: 'Identifier',
        name: '__template_expressions__',
        ...base,
      },
      value: {
        type: 'FunctionExpression',
        id: null,
        params: [],
        body: {
          type: 'BlockStatement',
          body: statements,
          ...base,
        },
        async: false,
        generator: false,
        ...base,
      },
      kind: 'method',
      static: false,
      computed: false,
      ...base,
    };
  }

  function traverse(node: any, parent: any = null, key: string | null = null, context = 'unknown'): any {
    if (!node || typeof node !== 'object') {
      return node;
    }

    // Handle arrays
    if (Array.isArray(node)) {
      return node.map((item, index) => traverse(item, node, String(index), context)).filter((item) => item !== null); // Remove nulls from arrays
    }

    // Check if this is a custom node
    if (node.type && isCustomNode(node)) {
      // Special handling for class bodies: extract JS expressions and create synthetic method
      // This allows semantic analysis to see variable usage in templates/custom nodes
      if (context === 'classBody' || (parent && parent.type === 'ClassBody' && key === 'body')) {
        const expressions = extractJSExpressionsFromCustomNode(node);
        if (expressions.length > 0) {
          return createSyntheticMethodForExpressions(expressions, node);
        }
        // No expressions found, skip it
        return null;
      }

      // Determine context for replacement
      const inferredContext =
        key === 'body'
          ? 'body'
          : key === 'expression'
            ? 'expression'
            : key === 'init' || key === 'test' || key === 'update'
              ? 'expression'
              : context;

      return createReplacement(node, inferredContext);
    }

    // Traverse known ESTree nodes
    const cloned: any = { ...node };

    for (const key in cloned) {
      if (key === 'parent') continue; // Skip parent references

      const value = cloned[key];

      if (value && typeof value === 'object') {
        // Determine context for child nodes
        // Special case: ClassBody.body should be treated as class body context, not statement body context
        const childContext =
          (cloned.type === 'ClassBody' && key === 'body')
            ? 'classBody'
            : key === 'body' || key === 'consequent' || key === 'alternate'
              ? 'body'
              : key === 'expression' || key === 'argument' || key === 'test'
                ? 'expression'
                : 'unknown';

        // Handle arrays specially - traverse and filter out nulls
        if (Array.isArray(value)) {
          const result = value.map((item, index) => traverse(item, cloned, key, childContext)).filter((item) => item !== null);
          cloned[key] = result;
        } else {
          const result = traverse(value, cloned, key, childContext);
          // If result is null and parent is ClassBody with body key, skip it
          if (result === null && cloned.type === 'ClassBody' && key === 'body') {
            // This shouldn't happen since we handle arrays above, but just in case
            continue;
          }
          cloned[key] = result;
        }
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
 * Strip custom nodes from an ESTree JSON string
 */
export function stripCustomNodesFromJSON(estreeJson: string, options?: StripOptions): string {
  const ast = JSON.parse(estreeJson);
  const { ast: strippedAst } = stripCustomNodes(ast, options);
  return JSON.stringify(strippedAst);
}

/**
 * Validate that an AST contains only known ESTree nodes
 */
export function validateESTreeAST(ast: any): {
  valid: boolean;
  customTypes: string[];
} {
  const customTypes = new Set<string>();

  function collect(node: any): void {
    if (!node || typeof node !== 'object') {
      return;
    }

    if (node.type && isCustomNode(node)) {
      customTypes.add(node.type);
    }

    if (Array.isArray(node)) {
      node.forEach(collect);
    } else {
      for (const key in node) {
        if (key !== 'parent' && key !== 'loc' && key !== 'range') {
          collect(node[key]);
        }
      }
    }
  }

  collect(ast);

  return {
    valid: customTypes.size === 0,
    customTypes: Array.from(customTypes),
  };
}

// Export types for TypeScript consumers
export type { StripOptions, StripResult };
