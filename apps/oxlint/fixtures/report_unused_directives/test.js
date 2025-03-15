// eslint-disable
const unusedVariable1 = 42;

// eslint-disable-next-line no-debugger
console.log('This is a test');

// eslint-enable

// eslint-disable-next-line no-console
debugger;

// eslint-disable-next-line no-unused-vars
const unusedVariable2 = 100;

function testFunction() {
    // eslint-disable-next-line no-console
    console.log('Inside test function');
}

testFunction();

// eslint-enable
