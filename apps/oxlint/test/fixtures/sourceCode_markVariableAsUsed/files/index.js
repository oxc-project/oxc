// `unusedTopLevel` is declared but never referenced in code,
// so `eslintUsed` should start as `false`.
const unusedTopLevel = "top";
const unusedTopLevel2 = "top2";

// `shadowedName` exists at module scope AND inside `inner`.
// Used to test that omitting `refNode` finds the module-scope one,
// and that a name can be in scope or out of scope depending on `refNode`.
const shadowedName = "module-level";

function outer(param) {
  // `nestedVar` is only in `outer`'s scope
  const nestedVar = param;
  const nestedVar2 = param + 1;

  function inner() {
    // Shadows module-level `shadowedName`
    const shadowedName = "inner-level";
    return shadowedName;
  }

  return nestedVar + inner();
}

export default outer(unusedTopLevel);
