// oxlint-disable no-unused-vars
let a;

// eslint-disable no-unused-vars
let b;

// oxlint-disable-next-line no-console
console.log("test");

// eslint-disable-next-line no-console
console.log("test2");

let c; // oxlint-disable-line no-unused-vars
let d; // eslint-disable-line no-unused-vars

// eslint-disable no-foo -- justification for disabling no-foo
// oxlint-disable no-bar -- justification for disabling no-bar
let e;
let f;

/* oxlint-disable no-unused-vars */
/* eslint-disable no-unused-vars */
let g;
let h;
/* oxlint-enable no-unused-vars */
/* eslint-enable no-unused-vars */

/* oxlint-disable -- rule-less oxlint block */
/* oxlint-enable -- rule-less oxlint enable */
/* eslint-disable -- rule-less eslint block */
/* eslint-enable -- rule-less eslint enable */
// oxlint-disable-next-line -- rule-less oxlint next-line
// eslint-disable-next-line -- rule-less eslint next-line
// oxlint-disable-line -- rule-less oxlint line
// eslint-disable-line -- rule-less eslint line
