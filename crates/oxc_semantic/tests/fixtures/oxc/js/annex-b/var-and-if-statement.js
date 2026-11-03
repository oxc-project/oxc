// A later `var` of the same name shares the hoisted function's binding.
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

function ifStatementFunctionThenLet() {
  {
    function g() {}
  }
  if (false) function g() {}
  let g = 1;
  return g;
}
