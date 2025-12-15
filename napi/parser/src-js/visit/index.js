import { createRequire } from "node:module";

// Lazy-loaded when first construct a `Visitor`
let walkProgram = null,
  addVisitorToCompiled,
  createCompiledVisitor,
  finalizeCompiledVisitor;

/**
 * Visitor class for traversing AST.
 */
export class Visitor {
  #compiledVisitor = null;

  constructor(visitor) {
    if (walkProgram === null) {
      const require = createRequire(import.meta.url);
      ({ walkProgram } = require("../../generated/visit/walk.js"));
      ({
        addVisitorToCompiled,
        createCompiledVisitor,
        finalizeCompiledVisitor,
      } = require("./visitor.js"));
    }

    const compiledVisitor = createCompiledVisitor();
    addVisitorToCompiled(visitor);
    const needsVisit = finalizeCompiledVisitor();
    if (needsVisit) this.#compiledVisitor = compiledVisitor;
  }

  /**
   * Visit AST.
   * @param program - The AST to visit.
   * @returns {undefined}
   */
  visit(program) {
    const compiledVisitor = this.#compiledVisitor;
    if (compiledVisitor !== null) walkProgram(program, compiledVisitor);
  }
}
