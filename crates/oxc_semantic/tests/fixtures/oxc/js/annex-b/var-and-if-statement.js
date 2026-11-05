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

// A `let` in the block containing an `if` prevents the branch function's Annex B var binding.
// The function has a separate symbol in its implicit branch scope.
function ifStatementFunctionAfterLet() {
  {
    let g;
    if (true) function g() {}
  }
  return g;
}
