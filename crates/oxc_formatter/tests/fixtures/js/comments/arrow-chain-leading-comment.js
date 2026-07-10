// Comment before a chain link, chain as grouped last call argument
foo(bar, (a) =>
  // c
  (b) => ({ x: 1 })
);

// Comment before a later chain link
foo(bar, (a) => (b) =>
  // c
  (c) => ({ x: 1 })
);

// Comment before the tail body
foo(bar, (a) => (b) =>
  // c
  ({ x: 1 })
);

// Array tail body
foo(bar, (a) =>
  // c
  (b) => [1]
);

// Comment before the tail body, assignment
const x = (a) => (b) =>
  // c
  ({ x: 1 });

// Comment before the tail body, callee
((a) => (b) =>
  // c
  ({ x: 1 }))();

// Comment before a chain link, assignment
const y = (a) =>
  // c
  (b) => ({ x: 1 });

// No comment: hugged layouts must stay unchanged
foo(bar, (a) => (b) => ({ someVeryLongPropertyName: 1, anotherVeryLongPropertyName: 2 }));
