use crate::snapshot;

#[test]
fn comment() {
    let cases = vec![
        r"
/**
 * Top level
 *
 * @module
 */

/** This is a description of the foo function. */
function foo() {
}

/**
 * Preserve newline
 */

/**
 * Represents a book.
 * @constructor
 * @param {string} title - The title of the book.
 * @param {string} author - The author of the book.
 */
function Book(title, author) {
}

/** Class representing a point. */
class Point {
    /**
     * Preserve newline
     */

    /**
     * Create a point.
     * @param {number} x - The x value.
     * @param {number} y - The y value.
     */
    constructor(x, y) {
    }

    /**
     * Get the x value.
     * @return {number} The x value.
     */
    getX() {
    }

    /**
     * Get the y value.
     * @return {number} The y value.
     */
    getY() {
    }

    /**
     * Convert a string containing two comma-separated numbers into a point.
     * @param {string} str - The string containing two comma-separated numbers.
     * @return {Point} A Point object.
     */
    static fromString(str) {
    }
}

/** Class representing a point. */
const Point = class {
}

/**
 * Shirt module.
 * @module my/shirt
 */

/** Button the shirt. */
exports.button = function() {
};

/** Unbutton the shirt. */
exports.unbutton = function() {
};

this.Book = function(title) {
    /** The title of the book. */
    this.title = title;
}
// https://github.com/oxc-project/oxc/issues/6006
export enum DefinitionKind {
  /**
   * Definition is a referenced variable.
   *
   * @example defineSomething(foo)
   */
  Reference = 'Reference',
  /**
   * Definition is a `ObjectExpression`.
   *
   * @example defineSomething({ ... })
   */
  Object = 'Object',
  /**
   * Definition is TypeScript interface.
   *
   * @example defineSomething<{ ... }>()
   */
  TS = 'TS',
}
export type TSTypeLiteral = {
    /**
     * Comment
     */
    foo: string
}
",
    ];

    snapshot("jsodc", &cases);
}
