// A bare `if` branch function has an implicit block. Its declaration must not capture a read
// of the enclosing block's lexical binding.
function branchReadsEnclosingBinding(a) {
  for (let g = 5; ;) {
    if (a) function g() {}
    return g;
  }
}

// The consequent and alternate have separate implicit scopes and function symbols, even though
// Annex B gives both a var binding when their branch is evaluated.
function branchesHaveDistinctBindings(a) {
  if (a) function g() { return 1; }
  else function g() { return 2; }
  return g;
}
