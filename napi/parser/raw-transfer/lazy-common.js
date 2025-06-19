'use strict';

// Unique token which is not exposed publicly.
// Used to prevent user calling class constructors.
const TOKEN = {};

/**
 * Throw error when restricted class constructor is called by user code.
 * @throws {Error}
 */
function constructorError() {
  throw new Error('Constructor is for internal use only');
}

module.exports = { TOKEN, constructorError };
