'use strict';

const { LEAF_NODE_TYPE_NAMES, createEmptyVisitor } = require('../generated/lazy/types.js');

// Getter for private `#visitor` property of `Visitor` class. Initialized in class body below.
let getCompiledVisitor;

/**
 * Visitor class, used to visit an AST.
 */
class Visitor {
  #visitor;

  /**
   * Create `Visitor`.
   *
   * Provide an object where keys are names of AST nodes you want to visit,
   * and values are visitor functions which receive AST node objects of that type.
   *
   * Keys can also be postfixed with `:exit` to visit when exiting the node, rather than entering.
   *
   * ```js
   * const visitor = new Visitor({
   *     BinaryExpression(binExpr) {
   *         // Do stuff when entering a `BinaryExpression`
   *     },
   *     'BinaryExpression:exit'(binExpr) {
   *         // Do stuff when exiting a `BinaryExpression`
   *     },
   * });
   * ```
   *
   * @constructor
   * @param {Object} visitor - Object defining visit functions for AST nodes
   * @returns {Visitor}
   */
  constructor(visitor) {
    this.#visitor = createCompiledVisitor(visitor);
  }

  static {
    getCompiledVisitor = visitor => visitor.#visitor;
  }
}

module.exports = { Visitor, getCompiledVisitor };

/**
 * Create compiled visitor object, keyed by node type name.
 *
 * Each property of object is one of:
 *
 * * No visitor for this type = `null`.
 * * Visitor for leaf node = visit function.
 * * Visitor for non-leaf node = object of form `{ enter, exit }`,
 *   where each property is either a visitor function or `null`.
 *
 * @param {Object} visitor - Visitor object from user
 * @returns {Object} - Compiled visitor object
 */
function createCompiledVisitor(visitor) {
  if (visitor === null || typeof visitor !== 'object') {
    throw new Error('`visitor` must be an object');
  }

  // Create empty compiled visitor
  const compiledVisitor = createEmptyVisitor();

  // Populate compiled visitor from provided object
  for (let name of Object.keys(visitor)) {
    const visitFn = visitor[name];
    if (typeof visitFn !== 'function') {
      throw new Error(`'${name}' property of \`visitor\` object is not a function`);
    }

    const isExit = name.endsWith(':exit');
    if (isExit) name = name.slice(0, -5);

    let existingVisitor = compiledVisitor[name];
    if (existingVisitor === null) {
      // Non-leaf node with no existing visitor
      compiledVisitor[name] = isExit
        ? { enter: null, exit: visitFn }
        : { enter: visitFn, exit: null };
    } else if (existingVisitor === false) {
      // Leaf node with no existing visitor
      compiledVisitor[name] = visitFn;
    } else if (typeof existingVisitor === 'object') {
      // Non-leaf node with an existing visitor. Add property for new
      if (isExit) {
        existingVisitor.exit = visitFn;
      } else {
        existingVisitor.enter = visitFn;
      }
    } else if (typeof existingVisitor === 'function') {
      // Leaf node with an existing visitor. Combine the 2.
      compiledVisitor[name] = isExit
        ? combineVisitFunctions(existingVisitor, visitFn)
        : combineVisitFunctions(visitFn, existingVisitor);
    } else {
      throw new Error(`Unknown node type '${name}' in \`visitor\` object`);
    }
  }

  // Set properties for leaf nodes with no visitor to `null`
  for (const name of LEAF_NODE_TYPE_NAMES) {
    if (compiledVisitor[name] === false) compiledVisitor[name] = null;
  }

  return compiledVisitor;
}

/**
 * Combine 2 visitor functions into 1.
 *
 * @param {function} visit1 - 1st visitor function
 * @param {function} visit2 - 2nd visitor function
 * @returns {function} - Combined visitor function
 */
function combineVisitFunctions(visit1, visit2) {
  return function(node) {
    visit1(node);
    visit2(node);
  };
}
