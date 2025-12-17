import {
  LEAF_NODE_TYPES_COUNT,
  NODE_TYPE_IDS_MAP,
  NODE_TYPES_COUNT,
} from "../generated/lazy/type_ids.js";

// Getter for private `#visitorsArr` property of `Visitor` class. Initialized in class body below.
let getVisitorsArrTemp;

/**
 * Visitor class, used to visit an AST.
 */
export class Visitor {
  #visitorsArr;

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
   * @class
   * @param {Object} visitor - Object defining visit functions for AST nodes
   * @returns {Visitor}
   */
  constructor(visitor) {
    this.#visitorsArr = createVisitorsArr(visitor);
  }

  static {
    getVisitorsArrTemp = (visitor) => visitor.#visitorsArr;
  }
}

export const getVisitorsArr = getVisitorsArrTemp;

/**
 * Create array of visitors, keyed by node type ID.
 *
 * Each element of array is one of:
 *
 * * No visitor for this type = `null`.
 * * Visitor for leaf node = visit function.
 * * Visitor for non-leaf node = object of form `{ enter, exit }`,
 *   where each property is either a visitor function or `null`.
 *
 * @param {Object} visitor - Visitors object from user
 * @returns {Array<Object|Function|null>} - Array of visitors
 */
function createVisitorsArr(visitor) {
  if (visitor === null || typeof visitor !== "object") {
    throw new Error("`visitor` must be an object");
  }

  // Create empty visitors array
  const visitorsArr = [];
  for (let i = NODE_TYPES_COUNT; i !== 0; i--) {
    visitorsArr.push(null);
  }

  // Populate visitors array from provided object
  for (let name of Object.keys(visitor)) {
    const visitFn = visitor[name];
    if (typeof visitFn !== "function") {
      throw new Error(`'${name}' property of \`visitor\` object is not a function`);
    }

    const isExit = name.endsWith(":exit");
    if (isExit) name = name.slice(0, -5);

    const typeId = NODE_TYPE_IDS_MAP.get(name);
    if (typeId === void 0) throw new Error(`Unknown node type '${name}' in \`visitor\` object`);

    if (typeId < LEAF_NODE_TYPES_COUNT) {
      // Leaf node. Store just 1 function.
      const existingVisitFn = visitorsArr[typeId];
      if (existingVisitFn === null) {
        visitorsArr[typeId] = visitFn;
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
    }

    if (isExit) {
      enterExit.exit = visitFn;
    } else {
      enterExit.enter = visitFn;
    }
  }

  return visitorsArr;
}

/**
 * Combine 2 visitor functions into 1.
 *
 * @param {function} visit1 - 1st visitor function
 * @param {function} visit2 - 2nd visitor function
 * @returns {function} - Combined visitor function
 */
function combineVisitFunctions(visit1, visit2) {
  return function (node) {
    visit1(node);
    visit2(node);
  };
}
