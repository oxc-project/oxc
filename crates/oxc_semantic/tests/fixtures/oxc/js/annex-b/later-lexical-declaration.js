// A lexical declaration after the block prevents the Annex B var binding.
function laterLet() {
  {
    function g() {}
  }
  let g = 1;
  return g;
}

function laterClass() {
  {
    function g() {}
  }
  class g {}
  return g;
}

function laterInEnclosingBlock() {
  {
    {
      function g() {}
    }
    let g;
  }
  return g;
}

function laterBlockFunctionInEnclosingBlock() {
  {
    {
      function g() {}
    }
    function g() {}
  }
  return g;
}
