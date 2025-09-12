// Unique token which is not exposed publicly.
// Used to prevent user calling class constructors.
export const TOKEN = {};

/**
 * Throw error when restricted class constructor is called by user code.
 * @throws {Error}
 */
export function constructorError() {
  throw new Error('Constructor is for internal use only');
}
