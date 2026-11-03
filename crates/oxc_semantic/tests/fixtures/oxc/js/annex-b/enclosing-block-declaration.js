// A declaration in a block between the function and its var scope prevents the Annex B var binding.
function enclosingLet() {
  {
    let g;
    {
      function g() {}
    }
  }
  return g;
}

function forHead() {
  for (let g; ; ) {
    function g() {}
  }
  return g;
}

// A simple catch parameter does not prevent it, but a destructured one does.
function simpleCatchParameter() {
  try {
  } catch (g) {
    {
      function g() {}
    }
  }
  return g;
}

function destructuredCatchParameter() {
  try {
  } catch ([g]) {
    {
      function g() {}
    }
  }
  return g;
}
