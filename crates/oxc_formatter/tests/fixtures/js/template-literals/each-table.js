// Formatted as a table
it.each`
  a    | b    | expected
  ${1} | ${1} | ${2}
  ${11} | ${22} | ${33}
`("returns $expected", ({ a, b, expected }) => {});

// Trailing comment stays inside the cell, even when the string contains `}`
it.each`
  stringified
  ${'{admin: 1}' /* invalid type for admin */}
`("should", ({ stringified, type }) => {});

// No header row: falls back to normal template printing, preserving the source
it.each`
    ${undefined}
    ${[]}
    ${[[]]}
  `("should return undefined when source is $source", ({ source }) => {});

// Cells whose expressions carry `expand_parent` sources (long member chain, call arguments)
// still measure as single-line columns; the softline removal wins
it.each`
  a | expected
  ${wrapper.instance().methods().filterProp().someOtherLongMethod().value()} | ${1}
  ${buildSomething(argumentNumberOne, argumentNumberTwo, argumentNumberThree)} | ${2}
`("returns $expected", ({ a, expected }) => {});
