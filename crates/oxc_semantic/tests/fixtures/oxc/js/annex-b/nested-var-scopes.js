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

// A hoist undone by an enclosing block's declaration, then a hoist in a nested function,
// then a sibling's hoist undone by a later declaration in the var scope.
function undoThenRehoist() {
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
