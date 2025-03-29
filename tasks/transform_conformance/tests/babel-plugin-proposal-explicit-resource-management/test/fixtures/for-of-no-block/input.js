// The arrow functions in this test case are to make sure that scopes are re-parented correctly
for (using x of (() => it)())
  doSomethingWith(x, () => {});
