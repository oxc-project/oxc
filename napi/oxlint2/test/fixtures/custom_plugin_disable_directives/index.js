// This var should trigger an error
var shouldError = 1;

// oxlint-disable-next-line no-var
var shouldBeDisabled = 2;

// eslint-disable-next-line no-debugger
debugger;

/* oxlint-disable-next-line no-var */
var anotherDisabled = 4;

// This var should trigger an error again
var shouldErrorAgain = 3;