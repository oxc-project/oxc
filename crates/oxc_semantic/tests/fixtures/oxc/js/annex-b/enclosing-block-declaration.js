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

// Each branch function has its own scope; reads in the catch still bind to its parameter.
function simpleCatchParameterIfStatement(x) {
  try {
  } catch (g) {
    if (x) function g() {}
    if (x) function g() {}
    g;
  }
  return g;
}

// The `var` already moved the catch parameter's symbol to the var scope.
function simpleCatchParameterVarIfStatement(x) {
  try {
  } catch (g) {
    var g;
    if (x) function g() {}
  }
  return g;
}

function destructuredCatchParameterIfStatement(x) {
  try {
  } catch ([g]) {
    if (x) function g() {}
  }
  return g;
}
