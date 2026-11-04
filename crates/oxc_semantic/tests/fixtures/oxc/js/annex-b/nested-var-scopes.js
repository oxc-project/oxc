// Lexical declarations only affect functions hoisted to their own var scope.
{
  function g() {}
}

function inner() {
  let g;
  {
    function h() {}
  }
  return [g, h];
}

const arrow = () => {
  {
    function h() {}
  }
  return h;
};

let h;
g();

// A function prevented by a later declaration in an enclosing block, then a function in a nested
// var scope, then a sibling prevented by a later declaration in the var scope.
function nestedVarScopeBetweenPreventedFunctions() {
  {
    {
      function g() {}
    }
    let g;
  }
  function nested() {
    {
      function g() {}
    }
  }
  {
    function g() {}
  }
  let g = 7;
  return g;
}
