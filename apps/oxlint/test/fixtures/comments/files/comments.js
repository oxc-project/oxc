const topLevelVariable1 = 1;
// Line comment 1
const topLevelVariable2 = 2; /* Block comment 1 */

/**
 * JSDoc comment
 */
export function topLevelFunction() {
  // Line comment 2
  /* Block comment 2 */
  let functionScopedVariable = topLevelVariable;
  /**
   * JSDoc comment 2
   */
  function nestedFunction() {
    // Line comment 3
    return functionScopedVariable;
  }
  return nestedFunction(); // Line comment 4
}

/* Block comment 3 */
const topLevelVariable3 = /* Block comment 4 */ 3;

const topLevelVariable4 = 4;
const topLevelVariable5 = 5;

// Line comment 5
// Line comment 6

const topLevelVariable6 = 6;
const topLevelVariable7 = 7;
