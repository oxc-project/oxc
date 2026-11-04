// A later `var` of the same name prevents the function's binding from moving to the var scope.
// The function stays bound in its block. Annex B makes it write the `var` when evaluated.
function laterVar() {
  {
    function g() {}
  }
  var g;
  return g;
}

// A function declaration that is an `if` branch does not prevent the Annex B var binding
// of a function in a nested block.
function ifStatementFunction() {
  {
    {
      function g() {
        return 1;
      }
    }
    if (false) function g() {
      return 2;
    }
  }
  return g();
}

// An `if` branch function that redeclares a `let` in the same block is that `let`'s symbol.
// It does not get a var binding, and neither does the `let`.
function ifStatementFunctionAfterLet() {
  {
    let g;
    if (true) function g() {}
  }
  return g;
}
