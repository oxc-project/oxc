// Examples of incorrect code for no-unsafe-call rule

declare const anyValue: any;

anyValue(); // unsafe call

anyValue(1, 2, 3); // unsafe call

const result = anyValue('hello'); // unsafe call

// Chained unsafe calls
anyValue().then().catch(); // unsafe