/* oxlint-disable no-console */
console.log("oxlint block");
/* oxlint-enable no-console */

console.log("oxlint line"); // oxlint-disable-line no-console

// oxlint-disable-next-line no-console
console.log("oxlint next line");

/* eslint-disable no-debugger */
debugger;
/* eslint-enable no-debugger */

debugger; // eslint-disable-line no-debugger

// eslint-disable-next-line no-debugger
debugger;

// Unused directive, reported only when inline configuration is respected.
// oxlint-disable-next-line no-console
const unusedDirective = 1;
