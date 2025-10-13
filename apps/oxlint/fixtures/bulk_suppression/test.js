console.log("This has violations");
var unused_variable = 123;
debugger;

function unusedFunction() {
    return null;
}

// eslint-disable-next-line no-console
console.log("This should still be disabled by comment");