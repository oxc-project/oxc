// eslint-disable-next-line no-console
console.log("eslint prefix is ignored");

// oxlint-disable-next-line no-console
console.log("oxlint prefix still suppresses");

// oxlint-disable-next-line no-debugger
console.log("oxlint unused");

// oxlint-disable-next-line no-console
debugger;

// eslint-enable
// oxlint-enable
