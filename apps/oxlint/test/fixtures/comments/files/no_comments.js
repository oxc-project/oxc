const topLevelVariable1 = 1;
const topLevelVariable2 = 2;

export function topLevelFunction() {
  let functionScopedVariable = topLevelVariable;
  function nestedFunction() {
    return functionScopedVariable;
  }
  return nestedFunction();
}

const topLevelVariable3 = 3;
const topLevelVariable4 = 4;
const topLevelVariable5 = 5;
const topLevelVariable6 = 6;
const topLevelVariable7 = 7;
