'use strict';

const {
  NODE_TYPE_IDS_MAP,
  NODE_TYPES_COUNT,
  LEAF_NODE_TYPES_COUNT,
  VISITOR_BITMAP_COUNT,
  createCompiledVisitorFromParts,
} = require('../generated/deserialize/lazy-types.js');

// Getter for private `#compiledVisitor` property of `Visitor` class. Initialized in class body below.
let getCompiledVisitor;

/**
 * Visitor class, used to visit an AST.
 */
class Visitor {
  #compiledVisitor;

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
    this.#compiledVisitor = createCompiledVisitor(visitor);
  }

  static {
    getCompiledVisitor = visitor => visitor.#compiledVisitor;
  }
}

module.exports = { Visitor, getCompiledVisitor };

// Array of bitmap integers.
// Reused in each call to `createCompiledVisitor`, because it's only temporary data.
const bitmaps = [];
for (let i = VISITOR_BITMAP_COUNT; i > 0; i--) {
  bitmaps.push(0);
}

/**
 * Create compiled visitor.
 *
 * Contains:
 *
 * 1. Array of visitors, keyed by node type ID.
 * 2. Series of 32-bit bitmaps, with a bit set for each type which has a visitor.
 *
 * e.g.:
 *
 * ```
 * {
 *   visitors: [ null, (node) => {}, null, null, ... ],
 *   bitmap0: 2, // 2nd bit is set because there's a visitor for 2nd node type
 *   bitmap1: 0,
 *   bitmap2: 0,
 *   bitmap3: 0,
 *   bitmap4: 0,
 *   bitmap5: 0,
 * }
 * ```
 *
 * Each element of `visitors` array is one of:
 *
 * * No visitor for this type = `null`.
 * * Visitor for leaf node = visit function.
 * * Visitor for non-leaf node = object of form `{ enter, exit }`,
 *   where each property is either a visitor function or `null`.
 *
 * @param {Object} visitor - Visitors object from user
 * @returns {Object} - Object of form `{ visitors, bitmap0, bitmap1, ... }`
 */
function createCompiledVisitor(visitor) {
  if (visitor === null || typeof visitor !== 'object') {
    throw new Error('`visitors` must be an object');
  }

  // Create empty visitors array
  const visitorsArr = [];
  for (let i = NODE_TYPES_COUNT; i !== 0; i--) {
    visitorsArr.push(null);
  }

  // Empty `bitmaps`
  bitmaps.fill(0);

  // Populate visitors array from provided object
  for (let name of Object.keys(visitor)) {
    const visitFn = visitor[name];
    if (typeof visitFn !== 'function') {
      throw new Error(`'${name}' property of \`visitors\` object is not a function`);
    }

    const isExit = name.endsWith(':exit');
    if (isExit) name = name.slice(0, -5);

    const typeId = NODE_TYPE_IDS_MAP.get(name);
    if (typeId === void 0) throw new Error(`Unknown node type '${name}' in \`visitors\` object`);

    if (typeId < LEAF_NODE_TYPES_COUNT) {
      // Leaf node. Store just 1 function.
      const existingVisitFn = visitorsArr[typeId];
      if (existingVisitFn === null) {
        visitorsArr[typeId] = visitFn;
        bitmaps[typeId >> 5] |= 1 << (typeId & 31);
      } else if (isExit) {
        visitorsArr[typeId] = combineVisitFunctions(existingVisitFn, visitFn);
      } else {
        visitorsArr[typeId] = combineVisitFunctions(visitFn, existingVisitFn);
      }
      continue;
    }

    let enterExit = visitorsArr[typeId];
    if (enterExit === null) {
      enterExit = visitorsArr[typeId] = { enter: null, exit: null };
      bitmaps[typeId >> 5] |= 1 << (typeId & 31);
    }

    if (isExit) {
      enterExit.exit = visitFn;
    } else {
      enterExit.enter = visitFn;
    }
  }

  return createCompiledVisitorFromParts(visitorsArr, bitmaps);
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
