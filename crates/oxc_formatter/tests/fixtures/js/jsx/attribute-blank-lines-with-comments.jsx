// Blank line, then comment, then next attribute
const a = (
  <Component
    first={1}

    // comment
    second={2}
  />
);

// Comment, then blank line, then next attribute
const b = (
  <Component
    first={1}
    // comment

    second={2}
  />
);

// Same-line trailing comment, then blank line, then next attribute
const c = (
  <Component
    first={1} // trailing

    second={2}
  />
);
